mod totp;
mod ui;

use std::io;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let secret = rpassword::prompt_password("请输入 Base32 密钥: ")?;

    let secret_bytes =
        totp::decode_secret(&secret).map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?;
    
    ui::run(&secret_bytes)?;

    Ok(())
}
