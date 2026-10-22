use std::{path::{Path, PathBuf}, rc::Rc, sync::{Arc, Mutex}};

use rfd::FileDialog;
use rusqlite::{Connection, types::Value};
use slint::{Model, ModelRc, SharedString, ToSharedString, VecModel, Weak, language::{StandardListViewItem, TableColumn}};


slint::include_modules!();


fn main()->Result<(), slint::PlatformError> {
    let ui = MainWindow::new()?;

    //let modello_colonne = Rc::new(VecModel::<TableColumn>::default());
    //let modello_righe = Rc::new(VecModel::<ModelRc<StandardListViewItem>>::default());

    let filePath:Arc<Mutex<Option<PathBuf>>> = Arc::new(Mutex::new(None));

    let handle_table_ui = ui.as_weak();
    let table_path = Arc::clone(&filePath);
    ui.on_setTable(move |table|{
        let ui = match handle_table_ui.upgrade() {
            Some(ui)=>ui,
            None => return
        };

        let pathGuard = table_path.lock().unwrap();
        let path = match pathGuard.as_ref() {
            Some(p)=>p,
            None=>return
        };

        let conn = match Connection::open(path){
            Ok(conn)=>conn,
            Err(e)=>{
                ui.invoke_showToast(e.to_shared_string());
                return;
            }
        };

        let mut stmt = match conn.prepare(format!("SELECT * FROM {}",table).as_str()){
            Ok(stmt)=>stmt,
            Err(e)=>{
                 ui.invoke_showToast(e.to_shared_string());
                return;
            }
        };

        let cols:Vec<TableColumn> = stmt.column_names().iter().map(|nome|{
            let mut col = TableColumn::default();
            col.title = nome.to_shared_string();
            col
        }).collect();
        ui.set_columns(ModelRc::new(VecModel::from(cols)));

        let cols_num = stmt.column_count();
        let mut rows = match stmt.query([]){
            Ok(rows)=>rows,
            Err(e)=>{

                 ui.invoke_showToast(e.to_shared_string());
                return;

            }
        };

        let mut rows_model:Vec<ModelRc<StandardListViewItem>> = vec![];

        while let Some(row) = match rows.next() {
            Ok(row)=>row,
            Err(e)=>{
                ui.invoke_showToast(e.to_shared_string());
                return;
            }
        } {
            let mut cells:Vec<StandardListViewItem> = vec![];

            for i in 0..cols_num  {
                let val:Value = match row.get(i) {
                    Ok(v)=>v,
                     Err(e)=>{
                        ui.invoke_showToast(e.to_shared_string());
                        return;
                    }
                };
                let txt = match val {
                    Value::Null => "NULL".to_string(),
                    Value::Integer(v)=>v.to_string(),
                    Value::Real(v)=>v.to_string(),
                    Value::Blob(v)=>format!("<BLOB: {} bytes>", v.len()),
                    Value::Text(v)=>v
                };
                let mut cell = StandardListViewItem::default();
                cell.text = SharedString::from(txt);
                cells.push(cell);
            }
            rows_model.push(ModelRc::new(VecModel::from(cells)));
        }
        ui.set_rows(ModelRc::new(VecModel::from(rows_model)));




    });

    let halde_ui = ui.as_weak();
    ui.on_open_file(move ||{
        let ui = match halde_ui.upgrade() {
            Some(ui)=>ui,
            None => return
        };

        let file = FileDialog::new()
                                        .set_title("Seleziona un database")
                                        .add_filter("db", &["db", "sqlite", "sql"])
                                        .pick_file();

        let f = match file {
            Some(f)=>f,
            None=>return
        };

        let mut path = filePath.lock().unwrap();
        *path = Some(f);

        let conn = match Connection::open(path.as_ref().unwrap()){
            Ok(conn)=>conn,
            Err(e)=>{
                ui.invoke_showToast(e.to_shared_string());
                return;
            }
        };

        let mut stmt = match conn.prepare("
            SELECT name
         FROM sqlite_master
         WHERE type = 'table'
           AND name NOT LIKE 'sqlite_%'
         ORDER BY name
        "){
            Ok(stmt)=>stmt,
            Err(e)=>{
                 ui.invoke_showToast(e.to_shared_string());
                return;
            }
        };
        let tables = match stmt.query_map([], |row|{
            row.get::<_, String>(0)
        }){
            Ok(tables)=>tables,
            Err(e)=>{
                 ui.invoke_showToast(e.to_shared_string());
                return;
            }
        };

        //let mut stmt2 = conn.prepare(format!("SELECT * FROM {}", tables[0]?).as_str());

        let str_tables:Vec<SharedString> = tables.map(|m|m.unwrap().to_shared_string()).collect();

        ui.set_tables(ModelRc::new(VecModel::from(str_tables)));


    });

    let ui_toast_weak = ui.as_weak();
    ui.on_showToast(move |message|{
        if let Some(ui) = ui_toast_weak.upgrade() {
            ui.set_error_msg(message);
            ui.set_toast_visible(true);
        }
    });

    let cell_weak = ui.as_weak();
    ui.on_cell_clicked(move |_y,_x|{
        let ui = match cell_weak.upgrade() {
            Some(ui)=>ui,
            None => return
        };
        let rows = ui.get_rows();
        if let Some(cells) = rows.row_data(_y as usize) {
            if let Some(cell) = cells.row_data(_x as usize)  {
                 ui.set_lateral_txt(cell.text);
            }
        }


    });

    //ui.invoke_showToast("Prova!".to_shared_string());

    ui.run()
}
