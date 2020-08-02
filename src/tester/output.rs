use std::collections::HashMap;
use std::error::Error;
// use std::io;
// use std::process;

use excel::*;
use simple_excel_writer as excel;

const STATIC_HEADERS: [&str; 5] = [
    "chip_number",
    "software",
    "chip_type",
    "flashed_id",
    "flashed_time",
];

const STATIC_CALC_HEADERS: [&str; 2] = ["db_vs_best", "PASS"];

const STATIC_CALC_ROW: [&str; 2] = [
    r#"=ROUNDDOWN({rssi_col}{current_row}-${top_5_db_avg}, -1)"#,
    r#"=IF(AND(OR({titlebars['To CDT']}{row}=\"yes\",{titlebars['To CDT']}{row}=\"n/a\"), OR({titlebars['From CDT']}{row}=\"yes\",{titlebars['From CDT']}{row}=\"n/a\"), {titlebars['Flashed']}{row}=\"yes\", {titlebars['dB vs best']}{row}>=-10), \"PASS\", \"NO PASS\")"#,
];

const STATIC_FINAL_CALC: [&str; 3] = [
    r#"=AVERAGEIF(B2:B58,CONCAT(">",MAX(B2:B58)-5))"#,
    r#"=COUNTIF(I$2:I58, "NO PASS")"#,
    r#"=H62/(ROW(I58)-1)"#,
];

pub fn save_report(headers: &Vec<String>, values: &Vec<HashMap<String, String>>) {
    // Create data headers
    let mut headers_final: Vec<String> = vec![];
    for el in STATIC_HEADERS.iter() {
        headers_final.push(el.to_string());
    }
    headers_final.extend_from_slice(&headers);

    // Add calculable headers
    let mut headers_final_w_calc = headers_final.clone();
    for el in STATIC_CALC_HEADERS.iter() {
        headers_final_w_calc.push(el.to_string());
    }

    let mut wb = Workbook::create("report.xlsx");
    let mut sheet = wb.create_sheet("Sheet1");

    // set column width
    for _ in headers_final_w_calc.iter() {
        sheet.add_column(Column { width: 10.0 });
    }

    let rssi_col = headers_final.clone().iter().position(|r| r == "rssi");
    let db_vs_best_col = (headers_final_w_calc.clone().iter().position(|r| r == "db_vs_best").unwrap() + 65) as u8 as char;

    wb.write_sheet(&mut sheet, |sw| {
        let mut header_row = row![];
        for el in headers_final_w_calc.iter() {
            header_row.add_cell(el.clone());
        }
        sw.append_row(header_row)?;
        let row_offset = 1;
        for (idx, hm) in values.iter().enumerate() {
            let mut data_row = row![];
            for el in headers_final.iter() {
                let val = hm.get(el);
                match val {
                    Some(v) => {
                        data_row.add_cell(v.to_string());
                    }
                    None => {
                        data_row.add_cell(String::new());
                    }
                }
            }

            let current_row = idx + row_offset + 1;
            let mut pass_string = vec![
                format!(r#"=IF(AND(C{}<>"","#, current_row), r#"), "PASS", "NO PASS")"#.to_owned()];
            match rssi_col {
                Some(col) => {
                    let rssi_col_i = (col + 65) as u8 as char ;
                    let rssi_cell_addr = format!("{}{}",rssi_col_i, current_row);
                    data_row.add_cell(format!(
                        r#"=ROUNDDOWN({rssi_cell_addr}-{top_5_db_avg}, -1)"#,
                        rssi_cell_addr = rssi_cell_addr,
                        top_5_db_avg = format!("$B${}", values.len() + 5)
                    ));

                    let rounddown_location = format!("{}{}", db_vs_best_col, current_row);
                    pass_string.insert(1, format!("{}>=-10", rounddown_location));
                    let pass_expr = pass_string.join(",");
                    data_row.add_cell(pass_expr);
                }
                None => {}
            }
            sw.append_row(data_row)?;
        }

        sw.append_blank_rows(3);

        match rssi_col {
            Some(col) => {
                let top_5_db_avg = format!(
                    r#"=AVERAGEIF({rssi_start}:{rssi_end},CONCAT(">",MAX({rssi_start}:{rssi_end})-5))"#,
                    rssi_start = format!("{}{}", (col + 65) as u8 as char, "2" ),
                    rssi_end = format!("{}{}", (col + 65) as u8 as char, values.len() + 1 ),
                );
                sw.append_row(row!["Top 5dB average:", top_5_db_avg]).unwrap();
            }
            None => {}
        }


        Ok(())
    })
    .expect("write excel error!");

    wb.close().expect("close excel error!");
}
