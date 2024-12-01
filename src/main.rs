use async_ssh2_tokio::client::{AuthMethod, Client, ServerCheckMethod};
use russh_sftp::{client::SftpSession, protocol::OpenFlags};
use tokio::fs::File;
use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;
use directories_next::UserDirs;


const USERNAME: &str = "mimeul";
//const REMOTE_RAW_DIR: &str = "/shares/sander.imm.uzh/MM/PRJEB57919/raw";
//const TB_PROFILER_SCRIPT: &str  = "/shares/sander.imm.uzh/MM/PRJEB57919/scripts/tbprofiler.sh";
//const TB_PROFILER_RESULTS_DIR: &str  = "/shares/sander.imm.uzh/MM/PRJEB57919/out/results";
//const DEFAULT_TEMPLATE_REMOTE: &str  = "/shares/sander.imm.uzh/MM/PRJEB57919/tb-profiler-templates/docx/default_template.docx";
//const USER_TEMPLATE_REMOTE_DIR: &str  = "/home/mimeul/shares/MM/PRJEB57919/template";
//const USER_TEMPLATE_REMOTE: &str  = "/shares/sander.imm.uzh/MM/PRJEB57919/template/user_template.docx";
//const ITOL_EXAMPLE_DATA: &str  = "/shares/sander.imm.uzh/MM/PRJEB57919/iTOL/example_data.zip";




#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {

    let remote_file_path = "/remote/path/to/file.docx";
    let local_file_path = "/local/path/to/file.docx";

    let key_path = match ssh_key_path() {
        Ok(path) => path,
        Err(e) => {
            eprintln!("Failed to get SSH key path: {}", e);
            return Ok(());
        }
    };
    let auth_method = AuthMethod::with_key_file(key_path, None);
    let client = Client::connect(
        ("130.60.24.133", 22),
        USERNAME,
        auth_method,
        ServerCheckMethod::NoCheck,
    )
    .await?;
    println!("Connected to the server");

    let channel = client.get_channel().await?;
    channel.request_subsystem(true, "sftp").await?;
    let sftp = SftpSession::new(channel.into_stream()).await?;

    let mut remote_file = sftp.open_with_flags(
        remote_file_path,
        OpenFlags::READ,
    )
    .await?;
    let mut local_file = File::create(local_file_path).await?;



    let mut buffer = [0u8; 4096];
    loop {
        let n = remote_file.read(&mut buffer).await?;
        if n == 0 {
            break; // End of file
        }
        local_file.write_all(&buffer[..n]).await?;
    }

    println!("File downloaded successfully to {}", local_file_path);

    Ok(())
}




fn ssh_key_path() -> Result<String, String> {
    if let Some(user_dirs) = UserDirs::new() {
        let path = user_dirs.home_dir().join(".ssh").join("id_rsa");
        if path.exists() {
            match path.to_str() {
                Some(path_str) => Ok(path_str.to_string()),
                None => Err("Failed to convert SSH key path to string".to_string()),
            }
        } else {
            Err(format!("SSH key file does not exist at: {:?}", path))
        }
    } else {
        Err("Failed to determine the user's home directory".to_string())
    }
}