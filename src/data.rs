use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct HsEntry {
    pub level:    String,
    pub hs:       String,
    pub vn:       String,
    pub en:       String,
    pub dvt:      String,
    pub nk_tt:    String,
    pub nk_ud:    String,
    pub vat:      String,
    pub acfta:    String,
    pub atiga:    String,
    pub ajcep:    String,
    pub vjepa:    String,
    pub akfta:    String,
    pub aanzfta:  String,
    pub aifta:    String,
    pub vkfta:    String,
    pub vcfta:    String,
    pub vneaeu:   String,
    pub cptpp:    String,
    pub ahkfta:   String,
    pub vncu:     String,
    pub evfta:    String,
    pub ukvfta:   String,
    pub vnlao:    String,
    pub vncam:    String,
    pub vifta:    String,
    pub rcept:    String,
    pub ttdb:     String,
    pub xk:       String,
    pub bvmt:     String,
    pub cshs:     String,
    pub giam_vat: String,
    pub chapter:  String,
}

impl HsEntry {
    pub fn digits(&self) -> String {
        self.hs.chars().filter(|c| c.is_ascii_digit()).collect()
    }

    pub fn formatted_code(&self) -> String {
        let d = self.digits();
        match d.len() {
            8 => format!("{}.{}.{}", &d[..4], &d[4..6], &d[6..8]),
            6 => format!("{}.{}", &d[..4], &d[4..6]),
            _ => self.hs.clone(),
        }
    }

    pub fn is_group_header(&self) -> bool {
        self.level == "0" || self.digits().len() == 4
    }

    pub fn chapter_num(&self) -> &str {
        self.chapter.splitn(2, " - ").next().unwrap_or("").trim()
    }

    pub fn chapter_name(&self) -> &str {
        self.chapter.splitn(2, " - ").nth(1).unwrap_or("").trim()
    }
}

use rusqlite::{params, Connection, Result};

pub fn init_db() -> Result<Connection> {
    let db_path = "hs_data.db";
    let is_new = !std::path::Path::new(db_path).exists();
    let mut conn = Connection::open(db_path)?;

    if is_new {
        conn.execute(
            "CREATE TABLE hs_entries (
                id INTEGER PRIMARY KEY,
                level TEXT, hs TEXT, vn TEXT, en TEXT, dvt TEXT, nk_tt TEXT, nk_ud TEXT, vat TEXT,
                acfta TEXT, atiga TEXT, ajcep TEXT, vjepa TEXT, akfta TEXT, aanzfta TEXT, aifta TEXT,
                vkfta TEXT, vcfta TEXT, vneaeu TEXT, cptpp TEXT, ahkfta TEXT, vncu TEXT, evfta TEXT,
                ukvfta TEXT, vnlao TEXT, vncam TEXT, vifta TEXT, rcept TEXT,
                ttdb TEXT, xk TEXT, bvmt TEXT, cshs TEXT, giam_vat TEXT, chapter TEXT,
                digits TEXT, vn_norm TEXT, en_norm TEXT
            )",
            [],
        )?;
        
        let tx = conn.transaction()?;
        let json = include_str!("../assets/hs_data.json");
        let entries: Vec<HsEntry> = serde_json::from_str(json).expect("Parse error");
        
        for e in entries {
            let digits = e.digits();
            let vn_n = normalize(&e.vn);
            let en_n = normalize(&e.en);
            tx.execute(
                "INSERT INTO hs_entries (
                    level, hs, vn, en, dvt, nk_tt, nk_ud, vat, acfta, atiga, ajcep, vjepa, akfta, aanzfta, aifta,
                    vkfta, vcfta, vneaeu, cptpp, ahkfta, vncu, evfta, ukvfta, vnlao, vncam, vifta, rcept,
                    ttdb, xk, bvmt, cshs, giam_vat, chapter, digits, vn_norm, en_norm
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15,
                          ?16, ?17, ?18, ?19, ?20, ?21, ?22, ?23, ?24, ?25, ?26, ?27,
                          ?28, ?29, ?30, ?31, ?32, ?33, ?34, ?35, ?36)",
                params![
                    e.level, e.hs, e.vn, e.en, e.dvt, e.nk_tt, e.nk_ud, e.vat, e.acfta, e.atiga, e.ajcep, e.vjepa, e.akfta, e.aanzfta, e.aifta,
                    e.vkfta, e.vcfta, e.vneaeu, e.cptpp, e.ahkfta, e.vncu, e.evfta, e.ukvfta, e.vnlao, e.vncam, e.vifta, e.rcept,
                    e.ttdb, e.xk, e.bvmt, e.cshs, e.giam_vat, e.chapter, digits, vn_n, en_n
                ]
            )?;
        }
        tx.commit()?;
    }
    
    Ok(conn)
}

pub fn get_total_count(conn: &Connection) -> usize {
    conn.query_row("SELECT COUNT(*) FROM hs_entries", [], |row| row.get::<_, i64>(0)).unwrap_or(0) as usize
}

pub fn search_db(conn: &Connection, q: &str) -> Vec<HsEntry> {
    if q.is_empty() { return vec![]; }
    
    let is_code = q.starts_with(|c: char| c.is_ascii_digit());
    let q_dig: String = q.chars().filter(|c| c.is_ascii_digit()).collect();
    let mut stmt;
    
    let query_str = "SELECT level, hs, vn, en, dvt, nk_tt, nk_ud, vat, acfta, atiga, ajcep, vjepa, akfta, aanzfta, aifta, vkfta, vcfta, vneaeu, cptpp, ahkfta, vncu, evfta, ukvfta, vnlao, vncam, vifta, rcept, ttdb, xk, bvmt, cshs, giam_vat, chapter FROM hs_entries";

    let mut rows = if is_code {
        stmt = conn.prepare(&format!("{} WHERE digits LIKE ? ORDER BY id ASC", query_str)).unwrap();
        stmt.query([format!("{}%", q_dig)]).unwrap()
    } else {
        let q_norm = normalize(q);
        let words: Vec<&str> = q_norm.split_whitespace().collect();
        let mut conditions = Vec::new();
        for _ in &words {
            conditions.push("(vn_norm LIKE ? OR en_norm LIKE ?)");
        }
        let cond_str = conditions.join(" AND ");
        let sql = format!("{} WHERE {} ORDER BY id ASC", query_str, cond_str);
        stmt = conn.prepare(&sql).unwrap();
        
        let mut params = Vec::new();
        for w in &words {
            let p = format!("%{}%", w);
            params.push(p.clone());
            params.push(p);
        }
        stmt.query(rusqlite::params_from_iter(params)).unwrap()
    };
    
    let mut results = Vec::new();
    while let Some(row) = rows.next().unwrap() {
        results.push(HsEntry {
            level: row.get(0).unwrap(), hs: row.get(1).unwrap(), vn: row.get(2).unwrap(), en: row.get(3).unwrap(), dvt: row.get(4).unwrap(),
            nk_tt: row.get(5).unwrap(), nk_ud: row.get(6).unwrap(), vat: row.get(7).unwrap(), acfta: row.get(8).unwrap(), atiga: row.get(9).unwrap(),
            ajcep: row.get(10).unwrap(), vjepa: row.get(11).unwrap(), akfta: row.get(12).unwrap(), aanzfta: row.get(13).unwrap(), aifta: row.get(14).unwrap(),
            vkfta: row.get(15).unwrap(), vcfta: row.get(16).unwrap(), vneaeu: row.get(17).unwrap(), cptpp: row.get(18).unwrap(), ahkfta: row.get(19).unwrap(),
            vncu: row.get(20).unwrap(), evfta: row.get(21).unwrap(), ukvfta: row.get(22).unwrap(), vnlao: row.get(23).unwrap(), vncam: row.get(24).unwrap(),
            vifta: row.get(25).unwrap(), rcept: row.get(26).unwrap(), ttdb: row.get(27).unwrap(), xk: row.get(28).unwrap(), bvmt: row.get(29).unwrap(),
            cshs: row.get(30).unwrap(), giam_vat: row.get(31).unwrap(), chapter: row.get(32).unwrap()
        });
    }
    results
}

/// Normalize Vietnamese text for accent-insensitive search
pub fn normalize(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        let mapped = match c {
            'à'|'á'|'â'|'ã'|'ä'|'å'|'ắ'|'ặ'|'ẳ'|'ẵ'|'ấ'|'ầ'|'ẩ'|'ẫ'|'ậ'|'ă' => 'a',
            'è'|'é'|'ê'|'ë'|'ế'|'ề'|'ể'|'ễ'|'ệ' => 'e',
            'ì'|'í'|'î'|'ï' => 'i',
            'ò'|'ó'|'ô'|'õ'|'ö'|'ố'|'ồ'|'ổ'|'ỗ'|'ộ'|'ớ'|'ờ'|'ở'|'ỡ'|'ợ'|'ơ' => 'o',
            'ù'|'ú'|'û'|'ü'|'ứ'|'ừ'|'ử'|'ữ'|'ự'|'ư' => 'u',
            'ý'|'ỳ'|'ỷ'|'ỹ'|'ỵ' => 'y',
            'đ' => 'd',
            c   => c,
        };
        out.push(mapped);
    }
    out.to_lowercase()
}
