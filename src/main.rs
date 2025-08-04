use std::env;
use std::path::Path;
use swhid::{SwhidComputer, SwhidError};

fn main() -> Result<(), SwhidError> {
    let args: Vec<String> = env::args().collect();
    
    if args.len() != 2 {
        eprintln!("Usage: {} <path>", args[0]);
        eprintln!("Compute SWHID for a file or directory");
        std::process::exit(1);
    }

    let path = &args[1];
    
    if path == "-" {
        // Read from stdin
        let mut data = Vec::new();
        std::io::Read::read_to_end(&mut std::io::stdin(), &mut data)?;
        
        let computer = SwhidComputer::new();
        let content = swhid::content::Content::from_data(data);
        let swhid = content.swhid();
        
        println!("{}", swhid);
    } else {
        let computer = SwhidComputer::new();
        let swhid = computer.compute_swhid(path)?;
        println!("{}", swhid);
    }

    Ok(())
} 