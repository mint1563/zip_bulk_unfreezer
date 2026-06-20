use std::fs::{self, File};
use std::io;
use std::path::Path;
use zip::ZipArchive;

fn main() -> io::Result<()> {
    // ZIPファイルを探す対象のフォルダ（今回はカレントディレクトリ）
    let scan_dir = Path::new("./zip");
    // 解凍先のフォルダ
    let output_dir = Path::new("unfreeze");

    // 出力先フォルダがなければ作成
    if !output_dir.exists() {
        fs::create_dir_all(output_dir)?;
    }

    println!("フォルダ内のZIPファイルをスキャン中...");

    // 1. 指定されたフォルダ内のファイルを走査
    for entry in fs::read_dir(scan_dir)? {
        let entry = entry?;
        let path = entry.path();

        // 拡張子が .zip のファイルかつ、出力先（unfreeze）自身でないことを確認
        if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("zip") {
            println!(
                "\n--- ZIPファイルを発見: {:?} ---",
                path.file_name().unwrap()
            );

            // 各ZIPファイルごとに専用の出力サブフォルダを作成 (例: unfreeze/archive1/)
            let zip_name = path.file_stem().unwrap();
            let target_output_dir = output_dir.join(zip_name);

            // 解凍処理を実行
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

/// ZIPファイルを解凍し、中に別のZIPがあればそれも再帰的に解凍する
fn extract_zip_and_nested(zip_path: &Path, output_dir: &Path) -> io::Result<()> {
    let file = File::open(zip_path)?;
    let mut archive = ZipArchive::new(file).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

    for i in 0..archive.len() {
        let mut file = archive
            .by_index(i)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        let outpath = match file.enclosed_name() {
            Some(path) => output_dir.join(path),
            None => continue,
        };

        if (*file.name()).ends_with('/') {
            // ディレクトリの作成
            fs::create_dir_all(&outpath)?;
        } else {
            // 親ディレクトリの存在確認と作成
            if let Some(p) = outpath.parent() {
                if !p.exists() {
                    fs::create_dir_all(p)?;
                }
            }

            // ファイルの書き出し
            let mut outfile = File::create(&outpath)?;
            io::copy(&mut file, &mut outfile)?;

            // 解凍されたファイルがさらにZIPだった場合（ネスト対応）
            if outpath.extension().and_then(|s| s.to_str()) == Some("zip") {
                println!(
                    "  └─ ネストされたZIPを展開します: {:?}",
                    outpath.file_name().unwrap()
                );

                let nested_output_dir = outpath.with_extension("");
                fs::create_dir_all(&nested_output_dir)?;

                // 再帰呼び出し
                if let Err(e) = extract_zip_and_nested(&outpath, &nested_output_dir) {
                    eprintln!("  ⚠️ 警告: 内包ZIPの解凍失敗: {}", e);
                }
            }
        }
    }

    Ok(())
}
