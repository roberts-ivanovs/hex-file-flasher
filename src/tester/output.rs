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

pub fn save_report(headers: &Vec<String>, values: &Vec<HashMap<String, String>>) {
    let mut headers_final: Vec<String> = vec![];

    for el in STATIC_HEADERS.iter() {
        headers_final.push(el.to_string());
    }
    headers_final.extend_from_slice(&headers);

    let mut wb = Workbook::create("report.xlsx");
    let mut sheet = wb.create_sheet("Sheet1");

    // set column width
    for _ in headers_final.iter() {
        sheet.add_column(Column { width: 10.0 });
    }

    wb.write_sheet(&mut sheet, |sw| {
        let mut header_row = row![];
        for el in headers_final.iter() {
            header_row.add_cell(el.clone());
        }
        sw.append_row(header_row)?;

        for hm in values.iter() {
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
            sw.append_row(data_row)?;
        }
        Ok(())
    })
    .expect("write excel error!");

    wb.close().expect("close excel error!");
}
