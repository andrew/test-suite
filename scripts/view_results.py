#!/usr/bin/env python3
"""
Generate a color-coded HTML table from SWHID harness test results.

This script reads the JSON results file and creates an HTML table showing:
- One row per test case
- One column per implementation
- Color-coded cells indicating test outcomes:
  - SKIP: Test was skipped (gray)
  - CONFORMANT: PASS with matching expected SWHID (green)
  - NON-CONFORMANT: PASS but wrong SWHID, or FAIL with expected (red)
  - EXECUTED_OK: PASS but no expected to compare (blue)
  - EXECUTED_ERROR: FAIL without expected (yellow/orange)
"""

import argparse
import json
import sys
from pathlib import Path
from typing import Dict, List, Optional, Tuple
from html import escape


def determine_cell_status(result: Dict, expected_swhid: Optional[str]) -> Tuple[str, str, Optional[str]]:
    """
    Determine the status, color, and content for a test result cell.
    
    Returns:
        Tuple of (status_label, html_color, cell_content)
    """
    status = result.get('status', 'UNKNOWN')
    swhid = result.get('swhid')
    error = result.get('error')
    
    if status == 'SKIPPED':
        return ('SKIP', '#888888', '')  # Gray - color only, no text
    
    if status == 'PASS':
        if expected_swhid:
            if swhid == expected_swhid:
                return ('CONFORMANT', '#90EE90', '')  # Light green - color only, no text
            else:
                # Non-conformant: show full wrong SWHID
                return ('NON-CONFORMANT', '#FF6B6B', swhid)  # Light red - full SWHID
        else:
            return ('EXECUTED_OK', '#87CEEB', '')  # Sky blue - color only
    
    if status == 'FAIL':
        if expected_swhid:
            # Non-conformant: show full wrong SWHID if available
            if swhid:
                return ('NON-CONFORMANT', '#FF6B6B', swhid)  # Full SWHID for discrepancy
            else:
                return ('NON-CONFORMANT', '#FF6B6B', '')  # Red - color only, no text
        else:
            return ('EXECUTED_ERROR', '#FFD700', '')  # Gold - color only, no text
    
    return ('UNKNOWN', '#FFFFFF', 'Unknown')


def get_error_summary(error: Optional[Dict]) -> str:
    """Extract a concise error summary from error dict."""
    if not error:
        return ''
    
    error_code = error.get('code', '')
    error_message = error.get('message', '')
    
    if error_code:
        return f"{error_code}: {error_message}"
    return error_message


def create_html_table(results_data: Dict) -> str:
    """Create an HTML table with color-coded results."""
    implementations = sorted([impl['id'] for impl in results_data.get('implementations', [])])
    tests = results_data.get('tests', [])
    
    if not implementations or not tests:
        return "<p>No data to display</p>"
    
    # Start HTML
    html = ['<!DOCTYPE html>', '<html>', '<head>', '<meta charset="UTF-8">']
    html.append('<title>SWHID Test Results</title>')
    html.append('<style>')
    html.append('''
        body {
            font-family: Arial, sans-serif;
            margin: 20px;
            background-color: #f5f5f5;
        }
        h1 {
            color: #333;
        }
        table {
            border-collapse: collapse;
            width: 100%;
            background-color: white;
            box-shadow: 0 2px 4px rgba(0,0,0,0.1);
        }
        th {
            background-color: #4CAF50;
            color: white;
            padding: 10px;
            text-align: left;
            font-weight: bold;
            position: sticky;
            top: 0;
            z-index: 10;
        }
        td {
            padding: 4px 8px;
            border: 1px solid #ddd;
            font-size: 10px;
            white-space: nowrap;
            overflow: hidden;
            text-overflow: ellipsis;
            max-width: 150px;
            min-width: 80px;
        }
        td.test-name {
            font-weight: bold;
            background-color: #f9f9f9;
            max-width: 300px;
        }
        td.expected {
            font-family: monospace;
            font-size: 9px;
            max-width: 200px;
        }
        tr:hover {
            background-color: #f5f5f5;
        }
        .legend {
            margin: 20px 0;
            padding: 15px;
            background-color: white;
            border-radius: 5px;
            box-shadow: 0 2px 4px rgba(0,0,0,0.1);
        }
        .legend-item {
            display: inline-block;
            margin: 5px 15px;
        }
        .color-box {
            display: inline-block;
            width: 20px;
            height: 20px;
            border: 1px solid #ccc;
            vertical-align: middle;
            margin-right: 5px;
        }
        .tooltip {
            position: relative;
            cursor: help;
        }
        .tooltip:hover::after {
            content: attr(title);
            position: absolute;
            left: 100%;
            top: 0;
            background-color: #333;
            color: white;
            padding: 5px 10px;
            border-radius: 3px;
            white-space: pre-wrap;
            z-index: 1000;
            min-width: 200px;
            font-size: 11px;
        }
    ''')
    html.append('</style>')
    html.append('</head>')
    html.append('<body>')
    html.append('<h1>SWHID Test Results</h1>')
    
    # Add legend
    html.append('<div class="legend">')
    html.append('<strong>Legend:</strong>')
    html.append('<div class="legend-item"><span class="color-box" style="background-color: #888888;"></span>SKIP - Test was skipped</div>')
    html.append('<div class="legend-item"><span class="color-box" style="background-color: #90EE90;"></span>CONFORMANT - PASS with matching expected SWHID</div>')
    html.append('<div class="legend-item"><span class="color-box" style="background-color: #FF6B6B;"></span>NON-CONFORMANT - Wrong SWHID or FAIL with expected</div>')
    html.append('<div class="legend-item"><span class="color-box" style="background-color: #87CEEB;"></span>EXECUTED_OK - PASS but no expected to compare</div>')
    html.append('<div class="legend-item"><span class="color-box" style="background-color: #FFD700;"></span>EXECUTED_ERROR - FAIL without expected</div>')
    html.append('</div>')
    
    # Start table
    html.append('<table>')
    html.append('<thead>')
    html.append('<tr>')
    html.append('<th>Test Case</th>')
    html.append('<th>Expected SWHID</th>')
    for impl in implementations:
        html.append(f'<th>{escape(impl)}</th>')
    html.append('</tr>')
    html.append('</thead>')
    html.append('<tbody>')
    
    # Add rows
    for test in tests:
        test_id = test.get('id', 'unknown')
        expected = test.get('expected', {})
        expected_swhid = expected.get('swhid')
        results = test.get('results', [])
        
        result_map = {r['implementation']: r for r in results}
        
        html.append('<tr>')
        
        # Test name
        html.append(f'<td class="test-name">{escape(test_id)}</td>')
        
        # Expected SWHID
        expected_display = escape(expected_swhid) if expected_swhid else ''
        html.append(f'<td class="expected">{expected_display}</td>')
        
        # Results per implementation
        for impl in implementations:
            result = result_map.get(impl)
            if not result:
                html.append('<td style="background-color: #f0f0f0;">N/A</td>')
            else:
                status_label, color, content = determine_cell_status(result, expected_swhid)
                
                # Build tooltip with full details
                tooltip_parts = [f"Status: {status_label}"]
                if result.get('swhid'):
                    tooltip_parts.append(f"SWHID: {result.get('swhid')}")
                if expected_swhid:
                    tooltip_parts.append(f"Expected: {expected_swhid}")
                if result.get('error'):
                    error_summary = get_error_summary(result.get('error'))
                    tooltip_parts.append(f"Error: {error_summary}")
                
                tooltip = '\n'.join(tooltip_parts)
                
                # Display content (for non-conformant, content already contains full SWHID)
                # For conformant/executed_ok, content is empty (color only)
                display_content = escape(str(content)).replace('\n', '<br>') if content else ''
                
                html.append(f'<td class="tooltip" style="background-color: {color};" title="{escape(tooltip)}">{display_content}</td>')
        
        html.append('</tr>')
    
    html.append('</tbody>')
    html.append('</table>')
    html.append('</body>')
    html.append('</html>')
    
    return '\n'.join(html)


def create_table_rich(results_data: Dict):
    """Create a rich table with color-coded results."""
    console = Console()
    
    # Get implementations and tests
    implementations = sorted([impl['id'] for impl in results_data.get('implementations', [])])
    tests = results_data.get('tests', [])
    
    if not implementations:
        console.print("[red]No implementations found in results[/red]")
        return None
    
    if not tests:
        console.print("[red]No test cases found in results[/red]")
        return None
    
    # Create table
    table = Table(title="SWHID Test Results", show_header=True, header_style="bold magenta")
    
    # Add columns
    table.add_column("Test Case", style="cyan", no_wrap=True)
    table.add_column("Expected", style="dim", max_width=20)
    
    for impl in implementations:
        table.add_column(impl, justify="center", max_width=25)
    
    # Add rows
    for test in tests:
        test_id = test.get('id', 'unknown')
        expected = test.get('expected', {})
        expected_swhid = expected.get('swhid')
        results = test.get('results', [])
        
        # Create result map by implementation
        result_map = {r['implementation']: r for r in results}
        
        # Build row
        row = [test_id]
        
        # Expected SWHID (shortened)
        expected_display = expected_swhid[:20] + '...' if expected_swhid and len(expected_swhid) > 20 else (expected_swhid or '')
        row.append(expected_display)
        
        # Results per implementation
        for impl in implementations:
            result = result_map.get(impl)
            if not result:
                cell_text = Text('N/A', style='dim white')
            else:
                status_label, color = determine_cell_status(result, expected_swhid)
                swhid = result.get('swhid')
                error = result.get('error')
                
                # Build cell content
                cell_parts = [status_label]
                
                if swhid:
                    swhid_short = swhid[:15] + '...' if len(swhid) > 15 else swhid
                    cell_parts.append(f"\n{swhid_short}")
                
                if error:
                    error_summary = get_error_summary(error)
                    if error_summary:
                        cell_parts.append(f"\n{error_summary}")
                
                cell_text = Text('\n'.join(cell_parts), style=color)
            
            row.append(cell_text)
        
        table.add_row(*row)
    
    return table


def create_table_basic(results_data: Dict) -> None:
    """Create a basic text table (fallback when rich is not available)."""
    implementations = sorted([impl['id'] for impl in results_data.get('implementations', [])])
    tests = results_data.get('tests', [])
    
    if not implementations or not tests:
        print("No data to display")
        return
    
    # Print header
    header = f"{'Test Case':<40} {'Expected':<25}"
    for impl in implementations:
        header += f" {impl:<25}"
    print(header)
    print("=" * len(header))
    
    # Print rows
    for test in tests:
        test_id = test.get('id', 'unknown')
        expected = test.get('expected', {})
        expected_swhid = expected.get('swhid')
        results = test.get('results', [])
        
        result_map = {r['implementation']: r for r in results}
        
        expected_display = expected_swhid[:24] if expected_swhid else ''
        row = f"{test_id:<40} {expected_display:<25}"
        
        for impl in implementations:
            result = result_map.get(impl)
            if not result:
                cell = 'N/A'
            else:
                status_label, _ = determine_cell_status(result, expected_swhid)
                swhid = result.get('swhid', '')
                error = result.get('error')
                
                cell_parts = [status_label]
                if swhid:
                    cell_parts.append(swhid[:15])
                if error:
                    error_summary = get_error_summary(error)
                    if error_summary:
                        cell_parts.append(error_summary[:20])
                
                cell = ' | '.join(cell_parts)
            
            row += f" {cell:<25}"
        
        print(row)


def print_legend(console: Optional = None):
    """Print a legend explaining the color codes."""
    if console:
        console.print("\n[bold]Legend:[/bold]")
        console.print("[dim white]SKIP[/dim white] - Test was skipped")
        console.print("[green]CONFORMANT[/green] - PASS with matching expected SWHID")
        console.print("[red]NON-CONFORMANT[/red] - PASS but wrong SWHID, or FAIL with expected")
        console.print("[blue]EXECUTED_OK[/blue] - PASS but no expected to compare")
        console.print("[yellow]EXECUTED_ERROR[/yellow] - FAIL without expected")
    else:
        print("\nLegend:")
        print("SKIP - Test was skipped")
        print("CONFORMANT - PASS with matching expected SWHID")
        print("NON-CONFORMANT - PASS but wrong SWHID, or FAIL with expected")
        print("EXECUTED_OK - PASS but no expected to compare")
        print("EXECUTED_ERROR - FAIL without expected")


def main():
    parser = argparse.ArgumentParser(
        description='Generate a color-coded table from SWHID harness test results',
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
Examples:
  %(prog)s results.json
  %(prog)s results.json --output results_table.html
        """
    )
    parser.add_argument(
        'results_file',
        type=str,
        help='Path to the JSON results file (e.g., results.json)'
    )
    parser.add_argument(
        '--output', '-o',
        type=str,
        help='Output HTML file (default: results.html in same directory as input)'
    )
    
    args = parser.parse_args()
    
    # Read results file
    results_path = Path(args.results_file)
    if not results_path.exists():
        print(f"Error: Results file not found: {results_path}", file=sys.stderr)
        sys.exit(1)
    
    try:
        with open(results_path, 'r') as f:
            results_data = json.load(f)
    except json.JSONDecodeError as e:
        print(f"Error: Invalid JSON in results file: {e}", file=sys.stderr)
        sys.exit(1)
    except Exception as e:
        print(f"Error reading results file: {e}", file=sys.stderr)
        sys.exit(1)
    
    # Generate HTML table
    html_content = create_html_table(results_data)
    
    if args.output:
        output_path = Path(args.output)
        with open(output_path, 'w', encoding='utf-8') as f:
            f.write(html_content)
        print(f"HTML table written to: {output_path}", file=sys.stderr)
    else:
        # Default to results.html if no output specified
        default_output = results_path.with_suffix('.html')
        with open(default_output, 'w', encoding='utf-8') as f:
            f.write(html_content)
        print(f"HTML table written to: {default_output}", file=sys.stderr)


if __name__ == '__main__':
    main()

