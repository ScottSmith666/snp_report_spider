use csv;
use regex::Regex;
use reqwest;
use scraper::{Html, Selector};
use std::fs;
use std::process::exit;
use std::time::Instant;
use tokio;
use tokio::runtime::Runtime;


const MAX_LOOP_OUT_TIME: i32 = 20;  // 重试次数

fn time_unit(seconds: f64) -> String {
    if seconds < 60 as f64 {
        String::from("秒")
    } else if seconds < 3600 as f64 {
        String::from("分钟")
    } else if seconds < (3600 * 24) as f64 {
        String::from("小时")
    } else {
        String::from("天")
    }
}

async fn get_snp_id(
    key: &String,
    posit_version: &String,
    proxy_port: &str,
) -> Result<String, reqwest::Error> {
    let get_snp_id_url = format!("https://www.ncbi.nlm.nih.gov/snp/?term={}", key);
    let mut snp_id = String::new();
    let mut client = reqwest::Client::new();
    if proxy_port != "0" {
        println!("Using proxy port: {}", proxy_port);
        let proxy = reqwest::Proxy::http(format!("http://127.0.0.1:{}", proxy_port))?;
        client = reqwest::Client::builder().proxy(proxy).build()?;
    } else {
        println!("No proxy...");
    }

    let response = client.get(get_snp_id_url)
        .header(
            "User-Agent",
            "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/138.0.0.0 Safari/537.36"
        ).send().await?;

    let body = response.text().await?;
    // 解析页面
    let document = Html::parse_document(&body);
    let snp_unit = Selector::parse(".supp").unwrap();
    let snp_answer = Selector::parse("dd").unwrap();
    let snp_id_selector = Selector::parse("a").unwrap();
    let elements = document.select(&snp_unit);
    for element in elements {
        // 在搜索到的snp结果间进行循环（可能搜到不止一条snp结果）
        let chr: Vec<_> = element.select(&snp_answer).nth(2).unwrap().text().collect();
        let grch = chr.join("");
        let grch_regex = Regex::new(r"(X|Y|MT|\d+)(\:)(\d+)").unwrap();
        let mut grch_version_p = "Posit version format ERROR!".to_string(); //
        if posit_version == "GRCh37" {
            grch_version_p = grch_regex
                .captures_iter(&grch)
                .nth(1)
                .iter()
                .nth(0)
                .unwrap()
                .get(0)
                .unwrap()
                .as_str()
                .to_string();
        } else if posit_version == "GRCh38" {
            grch_version_p = grch_regex
                .captures_iter(&grch)
                .nth(0)
                .iter()
                .nth(0)
                .unwrap()
                .get(0)
                .unwrap()
                .as_str()
                .to_string()
        } else {
            println!("{}", grch_version_p);
            exit(-1);
        }

        if grch_version_p != *key {
            continue;
        }

        snp_id = element
            .select(&snp_id_selector)
            .nth(0)
            .unwrap()
            .text()
            .nth(0)
            .unwrap()
            .to_string();
        println!(
            "Successfully catch snp id = {}, type = {}",
            snp_id, posit_version
        );
        break;
    }
    Ok(snp_id)
}

async fn get_alleles(
    snp_id: &String,
    from_gene: &String,
    writer: &mut csv::Writer<fs::File>,
    proxy_port: &str,
) -> Result<String, reqwest::Error> {
    let get_snp_alleles_maf_url =
        format!("https://www.ncbi.nlm.nih.gov/snp/{}#frequency_tab", snp_id);
    let mut client = reqwest::Client::new();
    if proxy_port != "0" {
        println!("Using proxy port: {}", proxy_port);
        let proxy = reqwest::Proxy::http(format!("http://127.0.0.1:{}", proxy_port))?;
        client = reqwest::Client::builder().proxy(proxy).build()?;
    } else {
        println!("No proxy...");
    }

    let response = client.get(get_snp_alleles_maf_url)
        .header(
            "User-Agent",
            "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/138.0.0.0 Safari/537.36"
        ).send().await?;
    let body = response.text().await?;
    // 解析页面
    let document = Html::parse_document(&body);
    let get_maf_table = Selector::parse("#popfreq_datatable").unwrap();
    let get_maf_table_row = Selector::parse(".chi_row").unwrap();
    for table in document.select(&get_maf_table) {
        for table_line in table.select(&get_maf_table_row) {
            let line_values = table_line.text().collect::<Vec<_>>();
            let _ = writer.write_record([
                snp_id,
                line_values[2],
                line_values[5],
                line_values[7],
                line_values[9],
                line_values[11],
                line_values[13],
                line_values[15],
                line_values[17],
                line_values[19],
                from_gene,
            ]);
        }
    }

    Ok(format!("获取{}等位基因数据成功！", snp_id))
}

fn main() {
    // 创建异步运行时
    let rt = Runtime::new().unwrap();

    // 命令行参数
    let mut args = std::env::args();
    if args.len() < 6 /* 6 is real number of parameter */ + 1
    /* 1 is program by self */
    {
        println!("缺少{}个参数，本程序无法运行!", 6 + 1 - args.len());
        println!(
            "用法：\n\t./get_snp_alleles <INPUT_FILE_PATH> <POSIT_VERSION> <OUTPUT_FILE_PATH> <KEY_COL_INDEX> <FROM_GENE_COL_INDEX> <USING_PROXY>"
        );
        exit(-1);
    }
    let input_file_path = args.nth(1).unwrap();
    let posit_version = args.nth(0).unwrap(); // posit version options: "GRCh37" or "GRCh38"
    if posit_version != "GRCh37" && posit_version != "GRCh38" {
        println!("无法识别的位置版本！");
        exit(-1);
    }
    let output_file_path = args.nth(0).unwrap();
    let key_col_index = args.nth(0).unwrap().parse::<usize>().unwrap() - 1;  // ncbi search snp keywords column index (1 is the first index)
    let from_gene_col_index = args.nth(0).unwrap().parse::<usize>().unwrap();
    let using_proxy = args.nth(0).unwrap().to_string(); // using proxy: "Y,<port>" e.g.: "Y,7890" or "N"

    let csv_input_file = fs::File::open(&input_file_path).unwrap();
    let csv_output_file = fs::File::create(&output_file_path).unwrap();
    let mut proxy_port = "0";
    if using_proxy != "N" {
        if !Regex::new(r"Y,\d+").unwrap().is_match(using_proxy.as_str()) {
            // 不符合"Y,<port>"格式
            println!("无法识别的proxy格式...");
            exit(-1);
        }
        proxy_port = using_proxy.split(",").collect::<Vec<&str>>()[1];
    }

    // Get input file total lines num
    let map_text = fs::read_to_string(&input_file_path).unwrap();
    let total_num: f64 = map_text.lines().collect::<Vec<_>>().len() as f64;

    let mut csv_reader = csv::Reader::from_reader(csv_input_file);
    let mut csv_writer = csv::Writer::from_writer(csv_output_file);
    // Write head of csv
    csv_writer
        .write_record([
            "SNP_ID",
            "Population",
            "Group",
            "Sample_Size",
            "Ref_Allele",
            "Alt_Allele",
            "Ref_HMOZ",
            "Alt_HMOZ",
            "HTRZ",
            "HWEP",
            "from_gene"
        ])
        .expect("Write failed!");

    let mut num = 1.0;
    let mut duration: f64 = 0.0;
    for line in csv_reader.records() {
        let start = Instant::now();
        let total_times = duration * (total_num - num);
        let unit = time_unit(total_times);
        let time_div_num = match unit.as_str() {
            "秒" => 1.0,
            "分钟" => 60.0,
            "小时" => 3600.0,
            "天" => 3600.0 * 24.0,
            _ => 0.0,
        };
        println!(
            "正在处理行：{:?}，进度：{:.0}/{:.0}, 剩余大约{:.2}{}...",
            line,
            num,
            total_num,
            total_times / time_div_num,
            unit
        );

        let line = line.unwrap();
        let search_key = &line[key_col_index].to_string();

        let from_gene = match from_gene_col_index {
            0 => {  // 不提供基因列
                println!("本次运行未提供基因列...");
                &String:: from("")
            }
            _ => {
                &line[from_gene_col_index - 1].to_string()
            }
        };

        // 通过正则表达式判断search_key的格式是否为snp id格式或chr:position格式
        if Regex::new(r"rs\d+").unwrap().is_match(search_key) {
            // 符合snp id格式
            let mut times = 0;
            loop {
                if times > MAX_LOOP_OUT_TIME {  // 如果超过最大重试次数则强制退出
                    println!("因多次重试失败，自动退出本轮任务，正在进入下一轮任务...");
                    break;
                }
                let finally = rt.block_on(get_alleles(search_key, &from_gene, &mut csv_writer, proxy_port));
                match finally {
                    Ok(msg) => {
                        println!("{}", msg);
                        break;
                    }
                    Err(e) => {
                        println!("获取等位基因结果出错，正在重试...");
                        eprintln!("{}", e);
                        times += 1;
                        continue;
                    }
                }
            }
            duration = Instant::now().duration_since(start).as_secs_f64(); // 程序一个循环总共秒数
            num += 1.0;
            continue;
        }

        let mut times = 1;
        loop {
            if times > MAX_LOOP_OUT_TIME {  // 如果超过最大重试次数则强制退出
                println!("因多次重试失败，自动退出本轮任务，正在进入下一轮任务...");
                break;
            }
            let result = rt.block_on(get_snp_id(&search_key, &posit_version, proxy_port));
            // 处理异步任务执行结果
            match result {
                Ok(snp_id) => {
                    let finally = rt.block_on(get_alleles(&snp_id, &from_gene, &mut csv_writer, proxy_port));
                    match finally {
                        Ok(msg) => {
                            println!("{}", msg);
                            break;
                        }
                        Err(e) => {
                            println!("获取等位基因结果出错，正在重试...");
                            eprintln!("{}", e);
                            times += 1;
                            continue;
                        }
                    }
                }
                Err(e) => {
                    println!("获取snp id出错，正在重试...");
                    eprintln!("{}", e);
                    times += 1;
                    continue;
                }
            }
        }
        duration = Instant::now().duration_since(start).as_secs_f64(); // 程序一个循环总共秒数
        num += 1.0;
    }
    csv_writer.flush().expect("Could not close file"); // 关闭文件
}
