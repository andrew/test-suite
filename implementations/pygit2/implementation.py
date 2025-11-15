"""
pygit2 SWHID Implementation Plugin

This module provides an interface to Git-based SWHID computation using pygit2.
"""

import os
import sys
import tempfile
import shutil
from pathlib import Path
from typing import Optional

from harness.plugins.base import SwhidImplementation, ImplementationInfo, ImplementationCapabilities

try:
    import pygit2
    PYGIT2_AVAILABLE = True
except ImportError:
    PYGIT2_AVAILABLE = False

class Implementation(SwhidImplementation):
    """pygit2 SWHID implementation plugin."""
    
    def get_info(self) -> ImplementationInfo:
        """Return implementation metadata."""
        return ImplementationInfo(
            name="pygit2",
            version="1.0.0",
            language="python",
            description="Git SWHID implementation using pygit2 (libgit2)",
            dependencies=["pygit2"]
        )
    
    def is_available(self) -> bool:
        """Check if pygit2 implementation is available."""
        return PYGIT2_AVAILABLE
    
    def get_capabilities(self) -> ImplementationCapabilities:
        """Return implementation capabilities."""
        return ImplementationCapabilities(
            supported_types=["cnt", "dir", "rev", "rel", "snp"],
            supported_qualifiers=["origin", "visit", "anchor", "path", "lines"],
            api_version="1.0",
            max_payload_size_mb=1000,
            supports_unicode=True,
            supports_percent_encoding=True
        )
    
    def compute_swhid(self, payload_path: str, obj_type: Optional[str] = None,
                     commit: Optional[str] = None, tag: Optional[str] = None) -> str:
        """Compute SWHID using pygit2 (libgit2)."""
        if not PYGIT2_AVAILABLE:
            raise RuntimeError("pygit2 library not available")
        
        payload_path = os.path.abspath(payload_path)
        
        if not os.path.exists(payload_path):
            raise FileNotFoundError(f"Payload not found: {payload_path}")
        
        # Auto-detect object type if not provided
        if obj_type is None:
            obj_type = self.detect_object_type(payload_path)
        
        # Skip snapshot objects as Git doesn't support them
        if obj_type == "snapshot":
            raise NotImplementedError("Git doesn't support snapshot objects")
        
        try:
            if obj_type == "content":
                return self._compute_content_swhid(payload_path)
            elif obj_type == "directory":
                return self._compute_directory_swhid(payload_path)
            elif obj_type == "revision":
                return self._compute_revision_swhid(payload_path, commit=commit)
            elif obj_type == "release":
                return self._compute_release_swhid(payload_path, tag=tag)
            else:
                raise ValueError(f"Unsupported object type: {obj_type}")
        except Exception as e:
            raise RuntimeError(f"Failed to compute Git SWHID: {e}")
    
    def _compute_content_swhid(self, file_path: str) -> str:
        """Compute content SWHID using pygit2 blob creation."""
        with open(file_path, 'rb') as f:
            content = f.read()
        
        # Create a temporary repository to use pygit2
        with tempfile.TemporaryDirectory() as temp_dir:
            repo_path = os.path.join(temp_dir, "repo")
            repo = pygit2.init_repository(repo_path)
            
            # Create blob object
            blob_id = repo.create_blob(content)
            blob_id_str = str(blob_id)
            
            return f"swh:1:cnt:{blob_id_str}"
    
    def _compute_directory_swhid(self, dir_path: str) -> str:
        """Compute directory SWHID using pygit2 tree creation."""
        # Create a temporary Git repository
        with tempfile.TemporaryDirectory() as temp_dir:
            repo_path = os.path.join(temp_dir, "repo")
            repo = pygit2.init_repository(repo_path)
            
            # Copy the directory contents to the repo, preserving symlinks
            target_path = os.path.join(repo_path, "target")
            if os.path.isdir(dir_path):
                shutil.copytree(dir_path, target_path, symlinks=True)
            else:
                # If it's a file, create a directory and put the file in it
                os.makedirs(target_path)
                shutil.copy2(dir_path, target_path)
            
            # Create tree for the target directory
            tree_id = self._create_git_tree_pygit2(repo, target_path)
            tree_id_str = str(tree_id)
            
            return f"swh:1:dir:{tree_id_str}"
    
    def _create_git_tree_pygit2(self, repo, dir_path):
        """Recursively create Git tree objects for a directory using pygit2."""
        tree_builder = repo.TreeBuilder()
        
        # Get all entries in the directory
        entries = []
        for item in os.listdir(dir_path):
            item_path = os.path.join(dir_path, item)
            
            if os.path.islink(item_path):
                # Handle symlink - Git stores symlinks as blob with mode GIT_FILEMODE_LINK
                link_target = os.readlink(item_path)
                blob_id = repo.create_blob(link_target.encode('utf-8'))
                entries.append((item, pygit2.GIT_FILEMODE_LINK, blob_id))
                
            elif os.path.isfile(item_path):
                # Handle file - check if executable
                with open(item_path, 'rb') as f:
                    content = f.read()
                
                blob_id = repo.create_blob(content)
                # Git uses GIT_FILEMODE_BLOB_EXECUTABLE for executable files
                import stat
                file_mode = os.stat(item_path).st_mode
                mode = pygit2.GIT_FILEMODE_BLOB_EXECUTABLE if (file_mode & stat.S_IEXEC) else pygit2.GIT_FILEMODE_BLOB
                entries.append((item, mode, blob_id))
                
            elif os.path.isdir(item_path):
                # Handle subdirectory
                sub_tree_id = self._create_git_tree_pygit2(repo, item_path)
                entries.append((item, pygit2.GIT_FILEMODE_TREE, sub_tree_id))
        
        # Sort entries (Git requires sorted tree entries)
        entries.sort(key=lambda x: x[0])
        
        # Add entries to tree builder
        for name, mode, oid in entries:
            tree_builder.insert(name, oid, mode)
        
        # Create tree object
        tree_id = tree_builder.write()
        
        return tree_id
    
    def _compute_revision_swhid(self, repo_path: str, commit: Optional[str] = None) -> str:
        """Compute revision SWHID using pygit2 commit hash."""
        # Open the repository
        repo = pygit2.Repository(repo_path)
        
        # Default to HEAD if no commit specified
        if commit is None:
            commit = "HEAD"
        
        # Resolve commit reference
        try:
            if commit == "HEAD":
                commit_obj = repo.head.peel(pygit2.Commit)
            elif len(commit) == 40:
                # Full SHA
                commit_obj = repo[commit]
            elif len(commit) == 7:
                # Short SHA - try to resolve by searching all refs and commits
                found = False
                # First try all refs
                for ref_name in repo.listall_references():
                    try:
                        ref = repo.lookup_reference(ref_name)
                        if ref.target.hex.startswith(commit):
                            commit_obj = repo[ref.target]
                            if isinstance(commit_obj, pygit2.Commit):
                                found = True
                                break
                    except:
                        continue
                
                # If not found in refs, try searching commits directly
                if not found:
                    try:
                        # Walk through all commits to find matching short SHA
                        for ref_name in repo.listall_references():
                            try:
                                ref = repo.lookup_reference(ref_name)
                                walker = repo.walk(ref.target, pygit2.GIT_SORT_TIME)
                                for commit_obj in walker:
                                    if commit_obj.id.hex.startswith(commit):
                                        found = True
                                        break
                                if found:
                                    break
                            except:
                                continue
                    except:
                        pass
                
                if not found:
                    raise ValueError(f"Could not resolve short SHA '{commit}'")
            else:
                # Try as ref name
                try:
                    ref = repo.lookup_reference(f"refs/heads/{commit}")
                    commit_obj = ref.peel(pygit2.Commit)
                except KeyError:
                    raise ValueError(f"Commit/ref '{commit}' not found")
        except (KeyError, ValueError) as e:
            raise ValueError(f"Could not resolve commit '{commit}': {e}")
        
        if not isinstance(commit_obj, pygit2.Commit):
            raise ValueError(f"Object '{commit}' is not a commit")
        
        commit_id = commit_obj.id.hex
        return f"swh:1:rev:{commit_id}"
    
    def _compute_release_swhid(self, repo_path: str, tag: Optional[str] = None) -> str:
        """Compute release SWHID using pygit2 tag hash."""
        if tag is None:
            raise ValueError("Tag name is required for release SWHID computation")
        
        # Open the repository
        repo = pygit2.Repository(repo_path)
        
        # Resolve tag reference
        try:
            tag_ref = repo.lookup_reference(f"refs/tags/{tag}")
        except KeyError:
            raise ValueError(f"Tag '{tag}' not found")
        
        # Get the tag object directly (not peeled) to check if it's an annotated tag
        # For annotated tags, the ref points to a Tag object
        # For lightweight tags, the ref points directly to a Commit
        try:
            # Try to get the tag object directly from the ref target
            tag_obj = repo[tag_ref.target]
            if isinstance(tag_obj, pygit2.Tag):
                # Annotated tag - use the tag object hash
                tag_id = tag_obj.id.hex
                return f"swh:1:rel:{tag_id}"
            else:
                # Lightweight tag - points to a commit, not a tag object
                raise ValueError(f"Tag '{tag}' is a lightweight tag, not an annotated tag. Releases require annotated tags.")
        except (KeyError, TypeError):
            # If we can't get the object directly, it might be a lightweight tag
            # Try peeling to see what it points to
            tag_obj = tag_ref.peel()
            if isinstance(tag_obj, pygit2.Tag):
                tag_id = tag_obj.id.hex
                return f"swh:1:rel:{tag_id}"
            else:
                raise ValueError(f"Tag '{tag}' is a lightweight tag, not an annotated tag. Releases require annotated tags.")
