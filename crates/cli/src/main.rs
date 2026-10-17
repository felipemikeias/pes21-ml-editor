//! `pes21-ml` — command-line editor for eFootball PES 2021 Master League saves.

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use clap::{Parser, Subcommand, ValueEnum};
use pes21_ml_core::crypto::{Container, Game, ENCRYPTION_HEADER_SIZE, MAX_SAVE_SIZE};
use pes21_ml_core::master_league::{Edit, MAX_BUDGET_RAW};
use pes21_ml_core::money::{self, group_thousands, Currency};
use pes21_ml_core::save_file::{read_limited, save_edits, write_atomically, MlSave};
use pes21_ml_core::Error;

#[derive(Parser)]
#[command(
    name = "pes21-ml",
    version,
    about = "Editor open source de saves da Master League do eFootball PES 2021 / open-source PES 2021 Master League save editor"
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Clone, Copy, ValueEnum)]
enum CurrencyArg {
    Eur,
    Usd,
    Jpy,
    Gbp,
    Brl,
}

impl From<CurrencyArg> for Currency {
    fn from(c: CurrencyArg) -> Self {
        match c {
            CurrencyArg::Eur => Currency::Euro,
            CurrencyArg::Usd => Currency::UsDollar,
            CurrencyArg::Jpy => Currency::Yen,
            CurrencyArg::Gbp => Currency::Pound,
            CurrencyArg::Brl => Currency::BrazilianReal,
        }
    }
}

#[derive(Subcommand)]
enum Command {
    /// Mostra os valores do save / show the save's values
    Show {
        file: PathBuf,
        /// Saída em JSON / JSON output
        #[arg(long)]
        json: bool,
        #[arg(long, value_enum, default_value = "eur")]
        currency: CurrencyArg,
    },
    /// Altera valores (cria backup automaticamente) / change values (backs up automatically)
    Set {
        file: PathBuf,
        /// Orçamento de transferências (na moeda escolhida, ou bruto com --raw)
        #[arg(long)]
        transfer: Option<String>,
        /// Orçamento salarial (na moeda escolhida, ou bruto com --raw)
        #[arg(long)]
        salary: Option<String>,
        /// Seguidores do clube / club followers
        #[arg(long)]
        followers: Option<String>,
        /// Nome do técnico (máx. 45 bytes UTF-8) / manager name
        #[arg(long)]
        manager_name: Option<String>,
        /// Define os dois orçamentos em 9.999.999.900 € / sets both budgets to €9,999,999,900
        #[arg(long, conflicts_with_all = ["transfer", "salary"])]
        max_money: bool,
        /// Interpreta os valores de orçamento como valor bruto do save (sem ×100)
        #[arg(long)]
        raw: bool,
        #[arg(long, value_enum, default_value = "eur")]
        currency: CurrencyArg,
        /// Não cria backup (não recomendado) / skip the backup (not recommended)
        #[arg(long)]
        no_backup: bool,
        /// Só mostra o que seria alterado / only show what would change
        #[arg(long)]
        dry_run: bool,
    },
    /// Descriptografa para uma pasta (formato do pesXdecrypter) / decrypt to a folder
    Decrypt {
        file: PathBuf,
        /// Pasta de saída (não pode existir) / output folder (must not exist)
        out_dir: PathBuf,
    },
    /// Recriptografa uma pasta do `decrypt` / re-encrypt a folder made by `decrypt`
    Encrypt {
        in_dir: PathBuf,
        out_file: PathBuf,
        /// Sobrescreve o arquivo de saída se existir / overwrite the output file
        #[arg(long)]
        force: bool,
    },
}

/// Parses an integer amount, ignoring thousands separators (`.`, `,`, `_`, spaces).
fn parse_amount(s: &str) -> Result<u64, String> {
    money::parse_amount(s).ok_or_else(|| format!("valor inválido / invalid number: {s:?}"))
}

fn budget_to_raw(s: &str, raw: bool, currency: Currency) -> Result<u32, String> {
    let n = parse_amount(s)?;
    if raw {
        return u32::try_from(n).map_err(|_| format!("valor grande demais / too large: {s}"));
    }
    let r = currency
        .to_raw(n)
        .ok_or_else(|| format!("valor grande demais / too large: {s}"))?;
    if u64::from(r) * 100 != n && currency == Currency::Euro {
        eprintln!(
            "aviso: o save guarda múltiplos de 100 €; {s} será gravado como {} €",
            group_thousands(u64::from(r) * 100, '.')
        );
    }
    Ok(r)
}

fn json_string(s: &str) -> String {
    let mut out = String::from("\"");
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

fn show(file: &Path, json: bool, currency: Currency) -> Result<(), Error> {
    let save = MlSave::load(file)?;
    let v = &save.values;
    let c = &save.container;
    if json {
        println!(
            "{{\"game_version\":{},\"description\":[{}],\"transfer_budget_raw\":{},\"salary_budget_raw\":{},\"transfer_budget\":{},\"salary_budget\":{},\"currency\":{},\"club_followers\":{},\"manager_name\":{}}}",
            json_string(&c.game_version()),
            c.description_text().iter().map(|s| json_string(s)).collect::<Vec<_>>().join(","),
            v.transfer_budget,
            v.salary_budget,
            currency.display(v.transfer_budget),
            currency.display(v.salary_budget),
            json_string(currency.symbol()),
            v.club_followers,
            v.manager_name.as_deref().map_or("null".to_string(), json_string),
        );
        return Ok(());
    }
    let sym = currency.symbol();
    println!("Jogo / game:            {}", c.game_version());
    for line in c.description_text() {
        println!("                        {line}");
    }
    println!(
        "Técnico / manager:      {}",
        v.manager_name
            .as_deref()
            .unwrap_or("(não encontrado / not found)")
    );
    println!(
        "Transferências:         {sym} {}  (bruto/raw {})",
        group_thousands(currency.display(v.transfer_budget), '.'),
        v.transfer_budget
    );
    println!(
        "Salários:               {sym} {}  (bruto/raw {})",
        group_thousands(currency.display(v.salary_budget), '.'),
        v.salary_budget
    );
    println!(
        "Seguidores / followers: {}",
        group_thousands(u64::from(v.club_followers), '.')
    );
    Ok(())
}

const FILES: [&str; 6] = [
    "encryptHeader.dat",
    "header.dat",
    "description.dat",
    "logo.png",
    "data.dat",
    "version.txt",
];

/// Creates a new file (never overwriting), private on Unix.
fn write_new(path: &Path, data: &[u8]) -> Result<(), Error> {
    let mut options = fs::OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    options.open(path)?.write_all(data)?;
    Ok(())
}

fn decrypt(file: &Path, out_dir: &Path) -> Result<(), Error> {
    let bytes = read_limited(file)?;
    let c = Container::decrypt_any(&bytes)?;
    // Decrypted blocks are private data: the folder must be new (never overwrite
    // anything) and, on Unix, readable only by the current user.
    let mut dir = fs::DirBuilder::new();
    dir.recursive(false); // the parent folder must already exist
    #[cfg(unix)]
    {
        use std::os::unix::fs::DirBuilderExt;
        dir.mode(0o700);
    }
    dir.create(out_dir)?;
    let blocks: [&[u8]; 6] = [
        &c.encryption_header,
        &c.file_header,
        &c.description,
        &c.logo,
        &c.data,
        &c.serial,
    ];
    for (name, data) in FILES.iter().zip(blocks) {
        write_new(&out_dir.join(name), data)?;
    }
    write_new(
        &out_dir.join("game.txt"),
        format!("{:?}\n", c.game).as_bytes(),
    )?;
    println!(
        "{} → {} ({})",
        file.display(),
        out_dir.display(),
        c.game.name()
    );
    Ok(())
}

fn encrypt(in_dir: &Path, out_file: &Path, force: bool) -> Result<(), Error> {
    if out_file.exists() && !force {
        return Err(Error::Io(std::io::Error::new(
            std::io::ErrorKind::AlreadyExists,
            "o arquivo de saída já existe (use --force) / output exists (use --force)",
        )));
    }
    // Folders made by pesXdecrypter have no game.txt: default to PES 2021.
    let game = match fs::read_to_string(in_dir.join("game.txt")) {
        Ok(name) => Game::ALL
            .into_iter()
            .find(|g| format!("{g:?}") == name.trim())
            .ok_or(Error::UnknownKey)?,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Game::Pes2021,
        Err(e) => return Err(e.into()),
    };
    // Check the combined size before reading anything into memory.
    let mut total: u64 = 0;
    for name in FILES {
        total = total.saturating_add(fs::metadata(in_dir.join(name))?.len());
    }
    if total > MAX_SAVE_SIZE as u64 {
        return Err(Error::TooLarge(
            usize::try_from(total).unwrap_or(usize::MAX),
        ));
    }
    let mut blocks = Vec::with_capacity(6);
    for name in FILES {
        blocks.push(read_limited(&in_dir.join(name))?);
    }
    let mut enc = [0u8; ENCRYPTION_HEADER_SIZE];
    if blocks[0].len() != ENCRYPTION_HEADER_SIZE {
        return Err(Error::BadLayout);
    }
    enc.copy_from_slice(&blocks[0]);
    let mut it = blocks.into_iter().skip(1);
    let mut next = || it.next().unwrap_or_default();
    let c = Container {
        game,
        encryption_header: enc,
        file_header: next(),
        description: next(),
        logo: next(),
        data: next(),
        serial: next(),
    };
    let bytes = c.encrypt()?;
    Container::decrypt(&bytes, game)?; // sanity check
    write_atomically(out_file, &bytes, None)?;
    println!(
        "{} → {} ({})",
        in_dir.display(),
        out_file.display(),
        game.name()
    );
    Ok(())
}

fn run(cli: Cli) -> Result<(), String> {
    match cli.command {
        Command::Show {
            file,
            json,
            currency,
        } => show(&file, json, currency.into()).map_err(|e| e.to_string()),
        Command::Decrypt { file, out_dir } => decrypt(&file, &out_dir).map_err(|e| e.to_string()),
        Command::Encrypt {
            in_dir,
            out_file,
            force,
        } => encrypt(&in_dir, &out_file, force).map_err(|e| e.to_string()),
        Command::Set {
            file,
            transfer,
            salary,
            followers,
            manager_name,
            max_money,
            raw,
            currency,
            no_backup,
            dry_run,
        } => {
            let currency: Currency = currency.into();
            let mut edit = if max_money {
                Edit::max_money()
            } else {
                Edit::default()
            };
            if let Some(t) = transfer {
                edit.transfer_budget = Some(budget_to_raw(&t, raw, currency)?);
            }
            if let Some(s) = salary {
                edit.salary_budget = Some(budget_to_raw(&s, raw, currency)?);
            }
            if let Some(f) = followers {
                let n = parse_amount(&f)?;
                edit.club_followers = Some(
                    u32::try_from(n)
                        .map_err(|_| "seguidores demais / too many followers".to_string())?,
                );
            }
            edit.manager_name = manager_name;
            if edit.is_empty() {
                return Err(
                    "nada para alterar; veja --help / nothing to change; see --help".into(),
                );
            }
            edit.validate().map_err(|e| e.to_string())?;
            let current = MlSave::load(&file).map_err(|e| e.to_string())?.values;
            let fmt = |raw: u32| {
                format!(
                    "{} {} (raw {raw})",
                    currency.symbol(),
                    group_thousands(currency.display(raw), '.')
                )
            };
            if let Some(v) = edit.transfer_budget {
                println!(
                    "Transferências: {} → {}",
                    fmt(current.transfer_budget),
                    fmt(v)
                );
            }
            if let Some(v) = edit.salary_budget {
                println!(
                    "Salários:       {} → {}",
                    fmt(current.salary_budget),
                    fmt(v)
                );
            }
            if let Some(v) = edit.club_followers {
                println!("Seguidores:     {} → {}", current.club_followers, v);
            }
            if let Some(v) = &edit.manager_name {
                println!(
                    "Técnico:        {} → {}",
                    current.manager_name.as_deref().unwrap_or("?"),
                    v
                );
            }
            if edit.transfer_budget.is_some_and(|v| v > MAX_BUDGET_RAW)
                || edit.salary_budget.is_some_and(|v| v > MAX_BUDGET_RAW)
            {
                eprintln!("aviso: acima de 9.999.999.900 €; valor não testado no jogo / warning: above €9,999,999,900, untested in game");
            }
            if dry_run {
                println!("(--dry-run: nada foi gravado / nothing written)");
                return Ok(());
            }
            let report = save_edits(&file, &edit, !no_backup).map_err(|e| e.to_string())?;
            if !report.changed {
                println!(
                    "Nada mudou; o arquivo não foi gravado / nothing changed, file not written"
                );
                return Ok(());
            }
            if let Some(b) = report.backup {
                println!("Backup: {}", b.display());
            }
            println!(
                "Salvo e verificado / saved and verified: {}",
                report.path.display()
            );
            Ok(())
        }
    }
}

const NO_ARGS_NOTICE: &str = "\
Este é o pes21-ml, a versão de LINHA DE COMANDO do editor.
Para usar a interface gráfica, abra o programa \"pes21-ml-editor\".

This is pes21-ml, the COMMAND-LINE version of the editor.
For the graphical app, open \"pes21-ml-editor\" instead.
";

/// Started without arguments, most likely by double-clicking it in a file manager:
/// explain what this program is instead of flashing a usage error and exiting.
fn no_arguments() -> ExitCode {
    use clap::CommandFactory;
    use std::io::IsTerminal;
    println!("{NO_ARGS_NOTICE}");
    let _ = Cli::command().print_help();
    // A console opened just for us (e.g. by Explorer or Finder) would close at once.
    if std::io::stdin().is_terminal() {
        print!("\nPressione Enter para fechar / Press Enter to close… ");
        let _ = std::io::stdout().flush();
        let _ = std::io::stdin().read_line(&mut String::new());
    }
    ExitCode::from(2)
}

fn main() -> ExitCode {
    if std::env::args_os().len() <= 1 {
        return no_arguments();
    }
    match run(Cli::parse()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("erro / error: {e}");
            ExitCode::FAILURE
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn amounts() {
        assert_eq!(parse_amount("9.999.999.900"), Ok(9_999_999_900));
        assert_eq!(parse_amount("1,000"), Ok(1000));
        assert!(parse_amount("-5").is_err());
        assert!(parse_amount("12a").is_err());
        assert!(parse_amount("").is_err());
        assert_eq!(
            budget_to_raw("9.999.999.900", false, Currency::Euro),
            Ok(99_999_999)
        );
        assert_eq!(budget_to_raw("123", true, Currency::Euro), Ok(123));
        assert!(budget_to_raw("99999999999", true, Currency::Euro).is_err());
    }

    #[test]
    fn json_escaping() {
        assert_eq!(json_string("a\"b\\c\n"), "\"a\\\"b\\\\c\\u000a\"");
    }

    #[test]
    fn cli_definition_is_valid() {
        use clap::CommandFactory;
        Cli::command().debug_assert();
    }
}
