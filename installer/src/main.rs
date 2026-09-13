use axum::{extract::Json as JBody,http::{header,HeaderValue,StatusCode},response::{sse::{Event as SE,KeepAlive},Html,IntoResponse,Json,Response,Sse},routing::{get,post},Router};
use serde::{Deserialize,Serialize};
use std::{env,net::SocketAddr,path::PathBuf,process::Command};
use tokio::sync::mpsc;
const UI:&str=include_str!("../static/installer.html");
#[derive(Serialize)]
struct Prereqs{rust:bool,git:bool,rust_ver:String,git_ver:String,os:String,arch:String}
#[derive(Deserialize)]
struct InstallReq{install_dir:String,add_path:bool,shortcut:bool,models:Vec<String>,source_build:bool}
#[derive(Serialize,Clone)]
struct Prog{step:u8,total:u8,msg:String,pct:u8,done:bool,err:Option<String>}
fn cmd_ver(c:&str,a:&str)->(bool,String){
    Command::new(c).arg(a).output()
        .map(|o|(o.status.success(),String::from_utf8_lossy(&o.stdout).trim().into()))
        .unwrap_or((false,String::new()))
}
async fn ui()->Html<&'static str>{Html(UI)}
fn _asset(body:&'static [u8],ct:&'static str)->Response{
    let mut r=Response::new(body.to_vec().into());
    *r.status_mut()=StatusCode::OK;
    r.headers_mut().insert(header::CONTENT_TYPE,HeaderValue::from_static(ct));
    r.headers_mut().insert(header::CACHE_CONTROL,HeaderValue::from_static("no-store"));
    r
}
async fn serve_dual_css()->impl IntoResponse{_asset(include_str!("../../static/dual-skin.css").as_bytes(),"text/css; charset=utf-8")}
async fn serve_dual_js()->impl IntoResponse{_asset(include_str!("../../static/dual-skin.js").as_bytes(),"application/javascript; charset=utf-8")}
async fn serve_archivo()->impl IntoResponse{_asset(include_bytes!("../../static/fonts/archivo-var.woff2"),"font/woff2")}
async fn serve_archivo_ext()->impl IntoResponse{_asset(include_bytes!("../../static/fonts/archivo-var-ext.woff2"),"font/woff2")}
async fn prereqs()->Json<Prereqs>{
    let(rust,rust_ver)=cmd_ver("rustc","--version");
    let(git,git_ver)=cmd_ver("git","--version");
    Json(Prereqs{rust,git,rust_ver,git_ver,os:env::consts::OS.into(),arch:env::consts::ARCH.into()})
}
async fn install(JBody(req):JBody<InstallReq>)->Sse<impl futures::Stream<Item=Result<SE,std::convert::Infallible>>>{
    let(tx,mut rx)=mpsc::channel::<Prog>(64);
    tokio::spawn(async move{do_install(req,tx).await});
    let stream=async_stream::stream!{while let Some(p)=rx.recv().await{
        yield Ok(SE::default().json_data(p).unwrap());
    }};
    Sse::new(stream).keep_alive(KeepAlive::default())
}
async fn do_install(req:InstallReq,tx:mpsc::Sender<Prog>){
    let model_count=req.models.len()as u8;
    let total=3+model_count+(if req.add_path{1}else{0})+(if req.shortcut{1}else{0});
    let mut step=0u8;
    let send=|s:u8,t:u8,m:String,p:u8,d:bool,e:Option<String>,tx:mpsc::Sender<Prog>|async move{
        let _=tx.send(Prog{step:s,total:t,msg:m,pct:p,done:d,err:e}).await;
    };
    let idir=PathBuf::from(&req.install_dir);
    step+=1;
    send(step,total,"Preparing install directory...".into(),5,false,None,tx.clone()).await;
    if let Err(e)=std::fs::create_dir_all(&idir){
        send(step,total,"Failed".into(),0,true,Some(e.to_string()),tx.clone()).await;return;
    }
    step+=1;
    if req.source_build{
        send(step,total,"Cloning Amni-Code repository...".into(),10,false,None,tx.clone()).await;
        let repo_dir=idir.join("Amni-Code");
        let clone_ok=if repo_dir.join(".git").exists(){
            tokio::process::Command::new("git").args(["pull"]).current_dir(&repo_dir)
                .output().await.map(|o|o.status.success()).unwrap_or(false)
        }else{
            tokio::process::Command::new("git")
                .args(["clone","https://github.com/anmire/Amni-Code.git",repo_dir.to_str().unwrap_or(".")])
                .output().await.map(|o|o.status.success()).unwrap_or(false)
        };
        if !clone_ok{
            send(step,total,"Git clone/pull failed".into(),0,true,Some("Check git installation".into()),tx.clone()).await;return;
        }
        step+=1;
        send(step,total,"Building Amni-Code (cargo build --release)...".into(),20,false,None,tx.clone()).await;
        let build=tokio::process::Command::new("cargo").args(["build","--release"])
            .current_dir(&repo_dir).output().await;
        match build{
            Ok(o)if o.status.success()=>{
                send(step,total,"Build successful!".into(),40,false,None,tx.clone()).await;
            }
            Ok(o)=>{
                let stderr=String::from_utf8_lossy(&o.stderr).to_string();
                send(step,total,"Build failed".into(),0,true,Some(stderr),tx.clone()).await;return;
            }
            Err(e)=>{
                send(step,total,"Build error".into(),0,true,Some(e.to_string()),tx.clone()).await;return;
            }
        }
    }else{
        send(step,total,"Downloading pre-built binary...".into(),10,false,None,tx.clone()).await;
        let bin_name=if cfg!(windows){"amni.exe"}else{"amni"};
        let dl_url=format!("https://github.com/anmire/Amni-Code/releases/latest/download/{}",bin_name);
        let dest=idir.join(bin_name);
        match reqwest::get(&dl_url).await{
            Ok(resp)if resp.status().is_success()=>{
                match resp.bytes().await{
                    Ok(data)=>{
                        if let Err(e)=std::fs::write(&dest,&data){
                            send(step,total,"Write failed".into(),0,true,Some(e.to_string()),tx.clone()).await;return;
                        }
                        #[cfg(unix)]
                        {use std::os::unix::fs::PermissionsExt;let _=std::fs::set_permissions(&dest,std::fs::Permissions::from_mode(0o755));}
                    }
                    Err(e)=>{send(step,total,"Download failed".into(),0,true,Some(e.to_string()),tx.clone()).await;return;}
                }
            }
            Ok(resp)=>{
                send(step,total,"Download failed".into(),0,true,Some(format!("HTTP {}",resp.status())),tx.clone()).await;return;
            }
            Err(e)=>{send(step,total,"Network error".into(),0,true,Some(e.to_string()),tx.clone()).await;return;}
        }
        step+=1;
        send(step,total,"Binary downloaded".into(),30,false,None,tx.clone()).await;
    }
    let base_pct=40u8;
    for(i,model)in req.models.iter().enumerate(){
        step+=1;
        let pct=base_pct+((i as u8+1)*40/model_count.max(1));
        send(step,total,format!("Downloading model: {}...",model),pct,false,None,tx.clone()).await;
        let local_dir=idir.join("models").join(model.split('/').last().unwrap_or(model));
        let _=std::fs::create_dir_all(&local_dir);
        let api_url=format!("https://huggingface.co/api/models/{}/tree/main",model);
        let files:Vec<HfFile>=match reqwest::get(&api_url).await.and_then(|r|Ok(r)){
            Ok(resp)=>resp.json().await.unwrap_or_default(),
            Err(_)=>{
                send(step,total,format!("Failed to list files for {}",model),pct,false,Some("Will retry on next launch".into()),tx.clone()).await;
                continue;
            }
        };
        for f in files.iter().filter(|f|f.rfilename.ends_with(".safetensors")||f.rfilename.ends_with(".json")||f.rfilename.ends_with(".txt")||f.rfilename.ends_with(".model")){
            let furl=format!("https://huggingface.co/{}/resolve/main/{}",model,f.rfilename);
            let fpath=local_dir.join(&f.rfilename);
            if fpath.exists()&&fpath.metadata().map(|m|m.len()).unwrap_or(0)==f.size{continue;}
            if let Some(parent)=fpath.parent(){let _=std::fs::create_dir_all(parent);}
            match reqwest::get(&furl).await{
                Ok(resp)if resp.status().is_success()=>{
                    if let Ok(data)=resp.bytes().await{let _=std::fs::write(&fpath,&data);}
                }
                _=>{}
            }
        }
    }
    if req.add_path{
        step+=1;
        send(step,total,"Adding to PATH...".into(),90,false,None,tx.clone()).await;
        let bin_dir=if req.source_build{idir.join("Amni-Code").join("target").join("release")}else{idir.clone()};
        setup_path(&bin_dir);
    }
    if req.shortcut{
        step+=1;
        send(step,total,"Creating shortcuts...".into(),95,false,None,tx.clone()).await;
        let bin=if req.source_build{idir.join("Amni-Code").join("target").join("release")}else{idir.clone()};
        create_shortcut(&bin);
    }
    send(step.max(total),total,"Installation complete!".into(),100,true,None,tx.clone()).await;
}
#[derive(Deserialize,Default)]
struct HfFile{
    #[serde(default)]rfilename:String,
    #[serde(default)]size:u64,
}
fn setup_path(bin_dir:&PathBuf){
    let dir_str=bin_dir.to_string_lossy().to_string();
    #[cfg(windows)]
    {
        use std::process::Command;
        let _=Command::new("powershell").args(["-Command",&format!(
            "$p=[Environment]::GetEnvironmentVariable('Path','User');if($p -notlike '*{}*'){{[Environment]::SetEnvironmentVariable('Path',\"$p;{}\".Trim(';'),'User')}}",
            dir_str.replace("\\","\\\\"),dir_str.replace("\\","\\\\")
        )]).output();
    }
    #[cfg(not(windows))]
    {
        let rc=dirs::home_dir().map(|h|if h.join(".zshrc").exists(){h.join(".zshrc")}else{h.join(".bashrc")}).unwrap_or_default();
        if let Ok(content)=std::fs::read_to_string(&rc){
            if !content.contains(&dir_str){
                let _=std::fs::write(&rc,format!("{}\nexport PATH=\"$PATH:{}\"\n",content,dir_str));
            }
        }
    }
}
fn create_shortcut(bin_dir:&PathBuf){
    #[cfg(windows)]
    {
        let exe=bin_dir.join("amni.exe");
        let desktop=dirs::desktop_dir().unwrap_or_else(||PathBuf::from("."));
        let lnk=desktop.join("Amni-Code.lnk");
        let _=Command::new("powershell").args(["-Command",&format!(
            "$s=(New-Object -COM WScript.Shell).CreateShortcut('{}');$s.TargetPath='{}';$s.Save()",
            lnk.to_string_lossy(),exe.to_string_lossy()
        )]).output();
    }
}
async fn open_link(JBody(url):JBody<serde_json::Value>)->Json<bool>{
    let u=url.as_str().unwrap_or("");
    let allowed=["https://rustup.rs","https://git-scm.com","https://www.python.org"];
    Json(allowed.iter().any(|a|u.starts_with(a))&&open::that(u).is_ok())
}
#[tokio::main]
async fn main(){
    let app=Router::new()
        .route("/",get(ui))
        .route("/assets/dual-skin.css",get(serve_dual_css))
        .route("/assets/dual-skin.js",get(serve_dual_js))
        .route("/assets/fonts/archivo-var.woff2",get(serve_archivo))
        .route("/assets/fonts/archivo-var-ext.woff2",get(serve_archivo_ext))
        .route("/api/prereqs",get(prereqs))
        .route("/api/install",post(install))
        .route("/api/open",post(open_link));
    let port=find_port();
    let addr=SocketAddr::from(([127,0,0,1],port));
    let listener=tokio::net::TcpListener::bind(addr).await.unwrap();
    let url=format!("http://127.0.0.1:{}",port);
    println!("Amni-Code Installer @ {}",url);
    tokio::spawn(async move{axum::serve(listener,app).await.ok();});
    tokio::time::sleep(std::time::Duration::from_millis(200)).await;
    use tao::event::{Event,WindowEvent};
    use tao::event_loop::{ControlFlow,EventLoop};
    use tao::window::WindowBuilder;
    use wry::WebViewBuilder;
    let evl=EventLoop::new();
    let win=WindowBuilder::new()
        .with_title("Amni-Code Installer")
        .with_inner_size(tao::dpi::LogicalSize::new(860.0,640.0))
        .build(&evl).unwrap();
    let _wv=WebViewBuilder::new()
        .with_url(&url)
        .with_devtools(cfg!(debug_assertions))
        .build(&win).unwrap();
    evl.run(move|event,_,cf|{
        *cf=ControlFlow::Wait;
        if let Event::WindowEvent{event:WindowEvent::CloseRequested,..}=event{*cf=ControlFlow::Exit;}
    });
}
fn find_port()->u16{
    (9100..9200).find(|p|std::net::TcpListener::bind(("127.0.0.1",*p)).is_ok()).unwrap_or(9100)
}
