use encoding_rs::SHIFT_JIS;
use std::fs::{self, File};
use std::io;
use std::path::Path;
use zip::ZipArchive;

fn main() -> io::Result<()> {
    let scan_dir = Path::new("./zip");
    let output_dir = Path::new("unfreeze");

    if !output_dir.exists() {
        fs::create_dir_all(output_dir)?;
    }

    println!("フォルダ内のZIPファイルをスキャン中（日本語文字化け対策版）...");

    for entry in fs::read_dir(scan_dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("zip") {
            let zip_name = path.file_stem().unwrap();
            let target_output_dir = output_dir.join(zip_name);

            // 【追加】すでに展開先フォルダが存在する場合はスキップ
            if target_output_dir.exists() {
                println!(
                    "スキップ: すでに展開済みです -> {:?}",
                    path.file_name().unwrap()
                );
                continue;
            }

            println!(
                "\n--- ZIPファイルを発見: {:?} ---",
                path.file_name().unwrap()
            );

            if let Err(e) = extract_zip_and_nested(&path, &target_output_dir) {
                eprintln!(
                    "エラー: {:?} の解凍に失敗しました: {}",
                    path.file_name().unwrap(),
                    e
                );
            }
        }
    }

    println!(
        "\nすべてのZIPファイルの解凍作業が完了しました。出力先: {:?}",
        output_dir
    );
    Ok(())
}

fn extract_zip_and_nested(zip_path: &Path, output_dir: &Path) -> io::Result<()> {
    let file = File::open(zip_path)?;
    let mut archive = ZipArchive::new(file).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

    for i in 0..archive.len() {
        let mut file = archive
            .by_index(i)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

        let sanitized_name = decode_filename(file.name_raw());
        if sanitized_name.is_empty() {
            continue;
        }

        let outpath = output_dir.join(&sanitized_name);

        if sanitized_name.ends_with('/') {
            fs::create_dir_all(&outpath)?;
        } else {
            if let Some(p) = outpath.parent() {
                if !p.exists() {
                    fs::create_dir_all(p)?;
                }
            }

            let mut outfile = File::create(&outpath)?;
            io::copy(&mut file, &mut outfile)?;

            if outpath.extension().and_then(|s| s.to_str()) == Some("zip") {
                let nested_output_dir = outpath.with_extension("");

                // 【追加】ネストされたZIPも展開先があればスキップ
                if nested_output_dir.exists() {
                    println!("  └─ スキップ（ネストZIP展開済み）: {}", sanitized_name);
                    continue;
                }

                println!("  └─ ネストされたZIPを展開します: {}", sanitized_name);
                fs::create_dir_all(&nested_output_dir)?;

                if let Err(e) = extract_zip_and_nested(&outpath, &nested_output_dir) {
                    eprintln!("  ⚠️ 警告: 内包ZIPの解凍失敗: {}", e);
                }
            }
        }
    }

    Ok(())
}

/// ZIP内の生のファイル名（バイト列）を安全に文字列へデコードする関数
fn decode_filename(raw_bytes: &[u8]) -> String {
    if let Ok(utf8_str) = std::str::from_utf8(raw_bytes) {
        return utf8_str.to_string();
    }

    let (res, _, has_errors) = SHIFT_JIS.decode(raw_bytes);
    if !has_errors {
        res.into_owned()
    } else {
        String::from_utf8_lossy(raw_bytes).into_owned()
    }
}
