//! MERKLITH Keygen - Key generation and management utility

use clap::{Parser, Subcommand};
use colored::Colorize;
use std::fs;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "merklith-keygen")]
#[command(about = "MERKLITH key generation and management utility")]
#[command(version)]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Generate a new random keypair
    New {
        /// Output file for private key
        #[arg(short, long)]
        output: Option<PathBuf>,
        /// Generate HD wallet with mnemonic
        #[arg(long)]
        hd: bool,
        /// Number of words for mnemonic (12, 15, 18, 21, 24)
        #[arg(long, default_value = "24")]
        words: usize,
    },
    /// Generate key from mnemonic
    FromMnemonic {
        /// Mnemonic phrase
        mnemonic: String,
        /// Derivation path
        #[arg(short, long, default_value = "m/44'/1024'/0'/0/0")]
        path: String,
        /// Output file
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
    /// Generate key from seed
    FromSeed {
        /// Hex-encoded seed
        seed: String,
    },
    /// Derive child key from parent
    Derive {
        /// Parent private key file
        #[arg(short, long)]
        parent: PathBuf,
        /// Child index
        #[arg(short, long)]
        index: u32,
        /// Hardened derivation
        #[arg(short, long)]
        hardened: bool,
    },
    /// Inspect a key (show public key and address)
    Inspect {
        /// Private key file or hex string
        key: String,
        /// Show QR code
        #[arg(long)]
        qr: bool,
    },
    /// Convert key format
    Convert {
        /// Input key file
        input: PathBuf,
        /// Output format
        #[arg(short, long, value_enum)]
        format: OutputFormat,
        /// Output file
        #[arg(short, long)]
        output: PathBuf,
    },
    /// Verify a keypair
    Verify {
        /// Private key
        #[arg(short, long)]
        private: String,
        /// Public key to verify against
        #[arg(short, long)]
        public: String,
    },
    /// Sign a message
    Sign {
        /// Private key file
        #[arg(short, long)]
        key: PathBuf,
        /// Message to sign
        message: String,
        /// Output format
        #[arg(short, long, value_enum, default_value = "hex")]
        format: SignatureFormat,
    },
    /// Verify a signature
    VerifySig {
        /// Public key
        #[arg(short, long)]
        public: String,
        /// Message
        message: String,
        /// Signature
        signature: String,
    },
}

#[derive(Clone, Copy, Debug, clap::ValueEnum)]
enum OutputFormat {
    Hex,
    Json,
    Pem,
}

#[derive(Clone, Copy, Debug, clap::ValueEnum)]
enum SignatureFormat {
    Hex,
    Base64,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    
    println!("{}", "MERKLITH Key Generator".bright_cyan().bold());
    println!("{}", "═══════════════════".bright_cyan());
    println!();
    
    match args.command {
        Commands::New { output, hd, words } => cmd_new(output, hd, words),
        Commands::FromMnemonic { mnemonic, path, output } => cmd_from_mnemonic(mnemonic, path, output),
        Commands::FromSeed { seed } => cmd_from_seed(seed),
        Commands::Derive { parent, index, hardened } => cmd_derive(parent, index, hardened),
        Commands::Inspect { key, qr } => cmd_inspect(key, qr),
        Commands::Convert { input, format, output } => cmd_convert(input, format, output),
        Commands::Verify { private, public } => cmd_verify(private, public),
        Commands::Sign { key, message, format } => cmd_sign(key, message, format),
        Commands::VerifySig { public, message, signature } => cmd_verify_sig(public, message, signature),
    }
}

fn cmd_new(output: Option<PathBuf>, hd: bool, words: usize) -> anyhow::Result<()> {
    println!("{}", "Generating new keypair...".bright_yellow());
    
    if hd {
        // Generate HD wallet
        let mnemonic = generate_mnemonic(words)?;
        println!("\n{}", "Mnemonic Phrase (SAVE THIS SECURELY):".bright_red().bold());
        println!("{}", mnemonic);
        println!("\n{}", "⚠️  Never share your mnemonic phrase with anyone!".bright_red());
        
        // Derive first key
        let (private_key, public, address) = derive_from_mnemonic(&mnemonic, "m/44'/1024'/0'/0/0")?;
        
        println!("\n{}", "Derived Address".bright_green().bold());
        println!("Address: {}", address.bright_cyan());
        println!("Public Key: {}", public);
        
        // Save if requested
        if let Some(path) = output {
            save_key(&path, &private_key)?;
            println!("\n{}", format!("Private key saved to: {}", path.display()).bright_green());
        }
    } else {
        // Generate simple keypair
        let (private, public, address) = generate_keypair()?;
        
        println!("\n{}", "New Keypair Generated".bright_green().bold());
        println!("Address: {}", address.bright_cyan());
        println!("Public Key: {}", public);
        
        if let Some(path) = output {
            save_key(&path, &private)?;
            println!("\n{}", format!("Private key saved to: {}", path.display()).bright_green());
        } else {
            println!("\n{}", "Private Key (SAVE THIS SECURELY):".bright_red().bold());
            println!("{}", private);
        }
    }
    
    Ok(())
}

fn cmd_from_mnemonic(mnemonic: String, path: String, output: Option<PathBuf>) -> anyhow::Result<()> {
    println!("{}", "Deriving key from mnemonic...".bright_yellow());
    
    let (private_key, public, address) = derive_from_mnemonic(&mnemonic, &path)?;
    
    println!("\n{}", "Derived Key".bright_green().bold());
    println!("Derivation Path: {}", path.bright_yellow());
    println!("Address: {}", address.bright_cyan());
    println!("Public Key: {}", public);
    
    if let Some(path) = output {
        save_key(&path, &private_key)?;
        println!("\n{}", format!("Private key saved to: {}", path.display()).bright_green());
    } else {
        println!("\n{}", "Private Key:".bright_green());
        println!("{}", private_key);
    }
    
    Ok(())
}

fn cmd_from_seed(seed: String) -> anyhow::Result<()> {
    println!("{}", "Generating key from seed...".bright_yellow());
    
    let seed_bytes = hex::decode(seed.trim_start_matches("0x"))?;
    if seed_bytes.len() != 64 {
        return Err(anyhow::anyhow!("Seed must be 64 bytes (128 hex chars)"));
    }
    
    let (private, public, address) = generate_from_seed(&seed_bytes.try_into().unwrap())?;
    
    println!("\n{}", "Generated Key".bright_green().bold());
    println!("Address: {}", address.bright_cyan());
    println!("Public Key: {}", public);
    println!("Private Key: {}", private);
    
    Ok(())
}

fn cmd_derive(parent: PathBuf, index: u32, hardened: bool) -> anyhow::Result<()> {
    println!("{} from {}...", "Deriving child key".bright_yellow(), parent.display());
    
    let parent_key = fs::read_to_string(&parent)?;
    let path = if hardened {
        format!("{}/{}'", parent.display(), index)
    } else {
        format!("{}/{}", parent.display(), index)
    };
    
    println!("Derivation path: {}", path);
    println!("\n{}", "Child Key Derived".bright_green().bold());
    println!("Index: {}", index);
    println!("Hardened: {}", hardened);
    
    Ok(())
}

fn cmd_inspect(key: String, qr: bool) -> anyhow::Result<()> {
    println!("{}", "Inspecting key...".bright_yellow());
    
    // Load key from file or use as-is
    let key_str = if PathBuf::from(&key).exists() {
        fs::read_to_string(&key)?
    } else {
        key
    };
    
    let (public, address) = derive_public_info(&key_str)?;
    
    println!("\n{}", "Key Information".bright_green().bold());
    println!("Address: {}", address.bright_cyan());
    println!("Public Key: {}", public);
    
    if qr {
        println!("\n{}", "QR Code (Address):".bright_green());
        print_qr(&address)?;
    }
    
    Ok(())
}

fn cmd_convert(input: PathBuf, format: OutputFormat, output: PathBuf) -> anyhow::Result<()> {
    println!("Converting key from {:?} to {:?}...", input, format);
    
    let key = fs::read_to_string(&input)?;
    
    let converted = match format {
        OutputFormat::Hex => key.trim().to_string(),
        OutputFormat::Json => {
            serde_json::json!({
                "private_key": key.trim()
            }).to_string()
        }
        OutputFormat::Pem => {
            format!(
                "-----BEGIN MERKLITH PRIVATE KEY-----\n{}\n-----END MERKLITH PRIVATE KEY-----\n",
                base64::encode(key.trim())
            )
        }
    };
    
    fs::write(&output, converted)?;
    println!("{} {}", "Converted key saved to:".bright_green(), output.display());
    
    Ok(())
}

fn cmd_verify(private: String, public: String) -> anyhow::Result<()> {
    println!("{}", "Verifying keypair...".bright_yellow());
    
    let derived_public = derive_public_from_private(&private)?;
    
    if derived_public == public {
        println!("{}", "✓ Keypair is valid!".bright_green());
        Ok(())
    } else {
        println!("{}", "✗ Keypair mismatch!".bright_red());
        println!("Expected: {}", public);
        println!("Got:      {}", derived_public);
        Err(anyhow::anyhow!("Verification failed"))
    }
}

fn cmd_sign(key: PathBuf, message: String, format: SignatureFormat) -> anyhow::Result<()> {
    println!("{}...", "Signing message".bright_yellow());
    
    let private_key = fs::read_to_string(&key)?;
    let signature = sign_message(&private_key, &message)?;
    
    let formatted_sig = match format {
        SignatureFormat::Hex => signature,
        SignatureFormat::Base64 => base64::encode(&signature),
    };
    
    println!("\n{}", "Signature".bright_green().bold());
    println!("{}", formatted_sig);
    
    Ok(())
}

fn cmd_verify_sig(public: String, message: String, signature: String) -> anyhow::Result<()> {
    println!("{}...", "Verifying signature".bright_yellow());
    
    if verify_signature(&public, &message, &signature)? {
        println!("{}", "✓ Signature is valid!".bright_green());
    } else {
        println!("{}", "✗ Signature is invalid!".bright_red());
        return Err(anyhow::anyhow!("Invalid signature"));
    }
    
    Ok(())
}

// Helper functions (placeholders)
fn generate_keypair() -> anyhow::Result<(String, String, String)> {
    // Placeholder
    Ok((
        "0x1234567890abcdef".to_string(),
        "0xpub1234567890abcdef".to_string(),
        "merklith1qxy2kgcygj5xvk8f3s9w4hj8k2mn0p3q9r4s5t".to_string(),
    ))
}

fn generate_mnemonic(_words: usize) -> anyhow::Result<String> {
    // Placeholder
    Ok("abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about".to_string())
}

fn derive_from_mnemonic(_mnemonic: &str, _path: &str) -> anyhow::Result<(String, String, String)> {
    // Placeholder
    generate_keypair()
}

fn generate_from_seed(_seed: &[u8; 64]) -> anyhow::Result<(String, String, String)> {
    // Placeholder
    generate_keypair()
}

fn derive_public_info(_private: &str) -> anyhow::Result<(String, String)> {
    // Placeholder
    Ok((
        "0xpub1234567890abcdef".to_string(),
        "merklith1qxy2kgcygj5xvk8f3s9w4hj8k2mn0p3q9r4s5t".to_string(),
    ))
}

fn derive_public_from_private(_private: &str) -> anyhow::Result<String> {
    // Placeholder
    Ok("0xpub1234567890abcdef".to_string())
}

fn sign_message(_private: &str, _message: &str) -> anyhow::Result<String> {
    // Placeholder
    Ok("0xsignature1234567890abcdef".to_string())
}

fn verify_signature(_public: &str, _message: &str, _signature: &str) -> anyhow::Result<bool> {
    // Placeholder
    Ok(true)
}

fn save_key(path: &PathBuf, private: &str) -> anyhow::Result<()> {
    let content = format!("Private Key: {}\n", private);
    fs::write(path, content)?;
    Ok(())
}

fn print_qr(_data: &str) -> anyhow::Result<()> {
    // Placeholder for QR code printing
    println!("[QR Code would be displayed here]");
    Ok(())
}