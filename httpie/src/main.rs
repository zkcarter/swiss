use std::collections::HashMap;
use std::str::FromStr;

use anyhow::{anyhow, Result};
use clap::{Args, Parser, Subcommand};
use colored::Colorize;
use mime::Mime;
use reqwest::{Client, header, Response, Url};
use syntect::{
    easy::HighlightLines,
    highlighting::{Style, ThemeSet},
    parsing::SyntaxSet,
    util::{as_24_bit_terminal_escaped, LinesWithEndings},
};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: SubCommand,
}

#[derive(Subcommand, Debug)]
enum SubCommand {
    Get(Get),
    Post(Post),
}

#[derive(Args, Debug)]
struct Get {
    #[arg(value_parser(parse_url))]
    url: String,
}

#[derive(Args, Debug)]
struct Post {
    #[arg(value_parser(parse_url))]
    url: String,
    #[arg(value_parser(parse_body))]
    body: Vec<Body>,
}

#[derive(Debug, Clone)]
struct Body {
    key: String,
    value: String,
}

impl FromStr for Body {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        let mut split = s.split("=");
        let err = || anyhow!(format!("Failed to parse {}", s));
        let key = (split.next().ok_or_else(err)?).to_string();
        let value = (split.next().ok_or_else(err)?).to_string();
        Ok(Self { key, value })
    }
}

fn parse_url(s: &str) -> Result<String> {
    let _url: Url = s.parse()?;
    Ok(s.into())
}

fn parse_body(s: &str) -> Result<Body> {
    s.parse()
}

async fn get(client: Client, args: &Get) -> Result<()> {
    let resp = client.get(&args.url).send().await?;
    Ok(print_resp(resp).await?)
}

async fn post(client: Client, args: &Post) -> Result<()> {
    let mut body = HashMap::new();
    for b in args.body.iter() {
        body.insert(&b.key, &b.value);
    }
    let resp = client.post(&args.url).json(&body).send().await?;
    Ok(print_resp(resp).await?)
}

async fn print_resp(resp: Response) -> Result<()> {
    print_status(&resp);
    print_headers(&resp);
    let mime = get_content_type(&resp);
    let body = resp.text().await?;
    print_body(mime, &body);
    Ok(())
}

fn print_status(resp: &Response) {
    let status = format!("{:?} {}", resp.version(), resp.status()).blue();
    println!("{}\n", status);
}

fn print_headers(resp: &Response) {
    for (name, value) in resp.headers() {
        println!("{}: {:?}", name.to_string().green(), value);
    }

    println!();
}

fn get_content_type(resp: &Response) -> Option<Mime> {
    resp.headers()
        .get(header::CONTENT_TYPE)
        .map(|v| v.to_str().unwrap().parse().unwrap())
}

fn print_body(m: Option<Mime>, body: &str) {
    match m {
        Some(v) if v == mime::APPLICATION_JSON => print_syntect(body, "json"),
        Some(v) if v == mime::TEXT_HTML => print_syntect(body, "html"),
        _ => println!("{}", body),
    }
}

fn print_syntect(s: &str, ext: &str) {
    let ps = SyntaxSet::load_defaults_newlines();
    let ts = ThemeSet::load_defaults();
    let syntax = ps.find_syntax_by_extension(ext).unwrap();
    let mut h = HighlightLines::new(syntax, &ts.themes["base16-ocean.dark"]);
    for line in LinesWithEndings::from(s) {
        let ranges: Vec<(Style, &str)> = h.highlight(line, &ps);
        let escaped = as_24_bit_terminal_escaped(&ranges[..], true);
        print!("{}", escaped);
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli: Cli = Cli::parse();
    let mut headers = header::HeaderMap::new();
    headers.insert("X-POWERED-BY", "zkLink".parse()?);
    headers.insert(header::USER_AGENT, "zkLink Httpie".parse()?);

    let client = Client::builder()
        .default_headers(headers)
        .build()?;
    let result = match cli.command {
        SubCommand::Get(ref args) => get(client, args).await?,
        SubCommand::Post(ref args) => post(client, args).await?
    };

    Ok(result)
}
