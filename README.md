# SNP Report Spider
### SNP Report Spider可方便地爬取csv表格中的SNP对应的不同人种的等位基因频率信息。
![](https://raw.githubusercontent.com/ScottSmith666/NCBI-SNP-Spider/refs/heads/main/imgs/ncbi.png)

## 1. 运行本程序
如果仅需运行本程序，可从GitHub Release下载对应的二进制文件，放置于本地软件目录，并部署环境变量，便可直接运行，无需在您的系统安装任何依赖。

二进制文件提供Windows x64版本和macOS arm64版本，如您有运行其他版本或/和需要自定义源码的需求，请自行下载源码编译，参考第2节。

## 2. 编译本程序用到的依赖
本项目依赖Rust软件（Cargo 1.89.0），请自行下载对应系统版本提前安装。

本项目所需依赖列于“Cargo.toml”中。

推荐使用JetBrains家的RustRover软件 https://www.jetbrains.com/rust/download/ ，个人可免费使用，且打开项目后会自动安装依赖，十分方便。

## 3. 用法
本程序为命令行软件，需要结合终端使用。用法如下：

```shell
srs <INPUT_FILE_PATH> <POSIT_VERSION> <OUTPUT_FILE_PATH> <KEY_COL_INDEX> <FROM_GENE_COL_INDEX> <USING_PROXY>
```

其中`<POSIT_VERSION>`参数为字符串型参数，只能为`GRCh37`或`GRCh38`，指定参数前请仔细确认自己的版本。

`<USING_PROXY>`参数为字符串型参数，是指定你是否在本程序中启用翻墙梯子以及翻墙梯子的端口号。由于ncbi是外网，有时可能速度慢，因此需要翻墙梯子。本参数的格式只能为`Y,梯子端口号`或`N`。举例：如`Y,7890`的含义是在本程序中启用翻墙梯子，并指定梯子的端口号是7890；`N`的含义是在本程序中禁用翻墙梯子。

参数`<INPUT_FILE_PATH>`与参数`<KEY_COL_INDEX>`和`<FROM_GENE_COL_INDEX>`一起使用。

`<KEY_COL_INDEX>`为整数型参数，其含义是作为爬取关键词的SNP ID在整个表格的第几列，从1开始计次，本程序支持“rs149308374”和“18:79410978”两种SNP ID格式。

`<FROM_GENE_COL_INDEX>`为整数型参数，其含义是作为爬取关键词的SNP对应的基因名在整个表格的第几列，从1开始计次，如不想提供基因名列次，可提供参数值“0”，这样结果中便不含基因名了。

`<INPUT_FILE_PATH>`为字符串型参数，其含义是用作输入的csv表格文件路径，其格式不要求固定，但要求必须包含SNP ID，格式支持“rs149308374”和“18:79410978”两种SNP ID格式。

我们以NCBI SNP Spider https://github.com/ScottSmith666/NCBI-SNP-Spider 的输出csv文件路径“/path/to/out.csv”举例：

out.csv的部分内容如下：

| rs_id | var_type | alleles | chr_grch38 | position_in_chr_grch38 | chr_grch37 | position_in_chr_grch37 | merged | merged_into_rs_id | in_which_gene | from_species |
|-------|----------|-------|------------| ---|------------| --- |--------|-------------------|---------------| --- |
| rs149308374  | SNV      | C>A,T | 18         | 79410978 | 18         | 77170978 | No     | None              | NFATC1        | Homo sapiens |
| rs149271669  | SNV      | C>G,T | 18         | 79527561 | 18         | 77287561 | No     | None              | NFATC1           | Homo sapiens |
| rs149224832  | SNV      | A>G   | 18         | 79498959 | 18         | 77258959 | No     | None              | NFATC1           | Homo sapiens |
| ...   | ...      | ...   | ...        | ... | ...        | ... | ...    | ...               | ...           | ... |
| rs1445697066   | DELINS   | 过长... | 18         | 79464614 | 18         | 77224614 | Yes    | rs57175022        | NFATC1           | Homo sapiens |
| ...   | ...      | ...   | ...        | ... | ...        | ... | ...    | ...               | ...           | ... |

根据表格内容，爬取关键词的SNP ID在上表第1列，爬取关键词的SNP对应的基因名在上表第10列

以启用端口号为7890的翻墙梯子、位置版本“GRCh37”、输出文件“/path/to/alleles_out.csv”为例：

```shell
srs /path/to/out.csv GRCh37 /path/to/alleles_out.csv 1 10 Y,7890
```

本程序具有重试功能，即程序在爬取某个SNP的信息意外出错时，可继续重试<=20次，直至成功爬取或者超出最大重试次数后，可进入下一个爬取SNP信息任务。

## 4. 爬取结果

| SNP_ID      | Population       | Group | Sample_Size | Ref_Allele | Alt_Allele | Ref_HMOZ | Alt_HMOZ | HTRZ | HWEP   | from_gene    |
|-------------|------------------|-------|-------------|-----------|------------|----------|----------|------|--------|--------------|
| rs149308374 | European         | Sub   | 9690        | C=1.0000  | A=0.0000, T=0.0000         | 1        | 0        | 0    | N/A    | NFATC1       |
| rs149308374 | African          | Sub | 2898        | C=1.0000  | A=0.0000, T=0.0000         | 1        | 0        | 0    | N/A | NFATC1 |
| rs149308374 | African Others   | Sub   | 114         | C=1.000   | A=0.000, T=0.000         | 1        | 0        | 0    | N/A | NFATC1 |
| rs149308374 | African American | Sub   | 2784        | C=1.0000  | A=0.0000, T=0.0000         | 1        | 0        | 0    | N/A | NFATC1 |
| rs149308374 | Asian            | Sub   | 112         | C=1.000   | A=0.000, T=0.000         | 1        | 0        | 0    | N/A | NFATC1 |
| rs149308374 | East Asian       | Sub   | 86          | C=1.00    | A=0.00, T=0.00         | 1        | 0        | 0    | N/A | NFATC1 |
| rs149308374 | Other Asian      | Sub   | 26          | C=1.00    | A=0.00, T=0.00        | 1        | 0        | 0    | N/A    | NFATC1          |
| rs149308374 | Latin American 1 | Sub | 146         | C=1.000   | A=0.000, T=0.000         | 1        | 0        | 0    | N/A | NFATC1 |
| rs149308374 | Latin American 2 | Sub | 610         | C=1.000          | A=0.000, T=0.000         | 1        | 0        | 0    | N/A | NFATC1 |
| rs149308374 | South Asian | Sub | 98          | C=1.00  | A=0.00, T=0.00         | 1        | 0        | 0    | N/A | NFATC1 |
| rs149308374 | Other | Sub | 496         | C=1.000  | A=0.000, T=0.000         | 1        | 0        | 0    | N/A | NFATC1 |
| ...         | ...              | ...   | ...         | ...       | ...        | ...      | ...      | ...  | ...    | ...          |

### 如果本程序有解决了一些你生活中的小烦恼，不妨请Scott Smith喝杯咖啡吧[Doge]
