//! Seed dataset JSONL format and directory validator (T-02-40).

use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;

use domain::{Decimal, Sku, Slug, TipoAtributo, TipoCodigo, UnidadMedida};
use serde_json::Value;

const MASTER_FILES: &[&str] = &[
    "categorias.jsonl",
    "marcas.jsonl",
    "atributos.jsonl",
    "productos.jsonl",
];

/// Single validation finding with location in the seed tree.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SeedIssue {
    pub archivo: String,
    pub linea: u32,
    pub campo: String,
    pub mensaje: String,
}

impl SeedIssue {
    fn new(
        archivo: impl Into<String>,
        linea: u32,
        campo: impl Into<String>,
        mensaje: impl Into<String>,
    ) -> Self {
        Self {
            archivo: archivo.into(),
            linea,
            campo: campo.into(),
            mensaje: mensaje.into(),
        }
    }
}

/// Validates a seed directory; returns all issues (empty if valid).
pub fn validate_dir(path: &Path) -> Vec<SeedIssue> {
    let mut issues = Vec::new();
    let mut categoria_slugs = HashSet::new();
    let mut marca_slugs = HashSet::new();
    let mut atributo_claves = HashSet::new();
    let mut product_skus = HashSet::new();

    for name in MASTER_FILES {
        let file_path = path.join(name);
        if !file_path.is_file() {
            issues.push(SeedIssue::new(
                *name,
                0,
                "",
                format!("missing required file {name}"),
            ));
        }
    }
    if !issues.is_empty() {
        return issues;
    }

    let mut categoria_rows: Vec<(u32, CategoriaRow)> = Vec::new();
    issues.extend(read_jsonl(
        &path.join("categorias.jsonl"),
        "categorias.jsonl",
        parse_categoria_row,
        &mut categoria_rows,
    ));
    let mut slug_first_line: HashMap<String, u32> = HashMap::new();
    for (linea, row) in &categoria_rows {
        if !slug_first_line.contains_key(&row.slug) {
            slug_first_line.insert(row.slug.clone(), *linea);
            categoria_slugs.insert(row.slug.clone());
        } else {
            issues.push(SeedIssue::new(
                "categorias.jsonl",
                *linea,
                "slug",
                "duplicate slug",
            ));
        }
    }
    for (linea, row) in &categoria_rows {
        if let Some(padre) = &row.padre
            && !categoria_slugs.contains(padre)
        {
            issues.push(SeedIssue::new(
                "categorias.jsonl",
                *linea,
                "padre",
                "unknown parent slug",
            ));
        }
    }

    let mut marca_rows: Vec<(u32, MarcaRow)> = Vec::new();
    issues.extend(read_jsonl(
        &path.join("marcas.jsonl"),
        "marcas.jsonl",
        parse_marca_row,
        &mut marca_rows,
    ));
    let mut marca_slug_lines: HashMap<String, u32> = HashMap::new();
    for (linea, row) in &marca_rows {
        if marca_slug_lines.contains_key(&row.slug) {
            issues.push(SeedIssue::new(
                "marcas.jsonl",
                *linea,
                "slug",
                "duplicate slug",
            ));
        } else {
            marca_slug_lines.insert(row.slug.clone(), *linea);
            marca_slugs.insert(row.slug.clone());
        }
    }

    let mut atributo_rows: Vec<(u32, AtributoRow)> = Vec::new();
    issues.extend(read_jsonl(
        &path.join("atributos.jsonl"),
        "atributos.jsonl",
        parse_atributo_row,
        &mut atributo_rows,
    ));
    let mut clave_lines: HashMap<String, u32> = HashMap::new();
    for (linea, row) in &atributo_rows {
        if clave_lines.contains_key(&row.clave) {
            issues.push(SeedIssue::new(
                "atributos.jsonl",
                *linea,
                "clave",
                "duplicate clave",
            ));
        } else {
            clave_lines.insert(row.clave.clone(), *linea);
            atributo_claves.insert(row.clave.clone());
        }
    }

    let mut product_rows: Vec<(u32, ProductoRow)> = Vec::new();
    issues.extend(read_jsonl(
        &path.join("productos.jsonl"),
        "productos.jsonl",
        parse_producto_row,
        &mut product_rows,
    ));
    let mut sku_lines: HashMap<String, u32> = HashMap::new();
    for (linea, row) in &product_rows {
        issues.extend(validate_producto_record(
            *linea,
            row,
            &categoria_slugs,
            &marca_slugs,
            &atributo_claves,
        ));
        if sku_lines.contains_key(&row.sku) {
            issues.push(SeedIssue::new(
                "productos.jsonl",
                *linea,
                "sku",
                "duplicate sku",
            ));
        } else {
            sku_lines.insert(row.sku.clone(), *linea);
            product_skus.insert(row.sku.clone());
        }
    }

    let relaciones_path = path.join("relaciones.jsonl");
    if relaciones_path.is_file() {
        issues.extend(validate_relaciones(&relaciones_path, &product_skus));
    }
    let sinonimos_path = path.join("sinonimos.jsonl");
    if sinonimos_path.is_file() {
        issues.extend(validate_sinonimos(&sinonimos_path));
    }
    let notas_path = path.join("notas.jsonl");
    if notas_path.is_file() {
        issues.extend(validate_notas(&notas_path, &product_skus));
    }
    let imagenes_path = path.join("imagenes");
    if imagenes_path.is_dir() {
        issues.extend(validate_imagenes_dir(&imagenes_path, &product_skus));
    }

    issues
}

fn read_jsonl<T, F>(
    path: &Path,
    archivo: &str,
    mut parse_row: F,
    out: &mut Vec<(u32, T)>,
) -> Vec<SeedIssue>
where
    F: FnMut(u32, &Value) -> Result<T, SeedIssue>,
{
    let mut issues = Vec::new();
    let content = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(e) => {
            issues.push(SeedIssue::new(
                archivo,
                0,
                "",
                format!("cannot read file: {e}"),
            ));
            return issues;
        }
    };
    for (idx, line) in content.lines().enumerate() {
        let linea = (idx + 1) as u32;
        if line.trim().is_empty() {
            continue;
        }
        match serde_json::from_str::<Value>(line) {
            Ok(value) => match parse_row(linea, &value) {
                Ok(row) => out.push((linea, row)),
                Err(issue) => issues.push(issue),
            },
            Err(_) => issues.push(SeedIssue::new(archivo, linea, "", "invalid json")),
        }
    }
    issues
}

struct CategoriaRow {
    slug: String,
    padre: Option<String>,
}

fn parse_categoria_row(linea: u32, value: &Value) -> Result<CategoriaRow, SeedIssue> {
    let archivo = "categorias.jsonl";
    let slug_raw = require_str(value, "slug", archivo, linea)?;
    if Slug::parse(slug_raw).is_err() {
        return Err(SeedIssue::new(
            archivo,
            linea,
            "slug",
            "invalid slug format",
        ));
    }
    let nombre = require_str(value, "nombre", archivo, linea)?;
    if nombre.trim().is_empty() {
        return Err(SeedIssue::new(
            archivo,
            linea,
            "nombre",
            "must not be empty",
        ));
    }
    let padre = match value.get("padre") {
        None | Some(Value::Null) => None,
        Some(Value::String(s)) => {
            if Slug::parse(s).is_err() {
                return Err(SeedIssue::new(
                    archivo,
                    linea,
                    "padre",
                    "invalid slug format",
                ));
            }
            Some(s.clone())
        }
        Some(_) => {
            return Err(SeedIssue::new(
                archivo,
                linea,
                "padre",
                "must be a slug string or null",
            ));
        }
    };
    Ok(CategoriaRow {
        slug: slug_raw.to_string(),
        padre,
    })
}

struct MarcaRow {
    slug: String,
}

fn parse_marca_row(linea: u32, value: &Value) -> Result<MarcaRow, SeedIssue> {
    let archivo = "marcas.jsonl";
    let slug_raw = require_str(value, "slug", archivo, linea)?;
    if Slug::parse(slug_raw).is_err() {
        return Err(SeedIssue::new(
            archivo,
            linea,
            "slug",
            "invalid slug format",
        ));
    }
    let nombre = require_str(value, "nombre", archivo, linea)?;
    if nombre.trim().is_empty() {
        return Err(SeedIssue::new(
            archivo,
            linea,
            "nombre",
            "must not be empty",
        ));
    }
    Ok(MarcaRow {
        slug: slug_raw.to_string(),
    })
}

struct AtributoRow {
    clave: String,
}

fn matches_clave_pattern(clave: &str) -> bool {
    if clave.is_empty() {
        return false;
    }
    let mut chars = clave.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    if !first.is_ascii_lowercase() {
        return false;
    }
    chars.all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
}

fn parse_atributo_row(linea: u32, value: &Value) -> Result<AtributoRow, SeedIssue> {
    let archivo = "atributos.jsonl";
    let clave = require_str(value, "clave", archivo, linea)?;
    if !matches_clave_pattern(clave) {
        return Err(SeedIssue::new(
            archivo,
            linea,
            "clave",
            "invalid clave format",
        ));
    }
    let etiqueta = require_str(value, "etiqueta", archivo, linea)?;
    if etiqueta.trim().is_empty() {
        return Err(SeedIssue::new(
            archivo,
            linea,
            "etiqueta",
            "must not be empty",
        ));
    }
    let tipo_raw = require_str(value, "tipo", archivo, linea)?;
    let tipo = TipoAtributo::parse_wire(tipo_raw)
        .map_err(|_| SeedIssue::new(archivo, linea, "tipo", "invalid tipo"))?;
    if tipo == TipoAtributo::Enum {
        match value.get("opciones") {
            Some(Value::Array(arr)) if !arr.is_empty() && arr.iter().all(|v| v.is_string()) => {}
            _ => {
                return Err(SeedIssue::new(
                    archivo,
                    linea,
                    "opciones",
                    "required for enum tipo",
                ));
            }
        }
    }
    Ok(AtributoRow {
        clave: clave.to_string(),
    })
}

#[derive(Debug)]
struct ProductoRow {
    sku: String,
    categoria: String,
    marca: Option<String>,
    precio_raw: String,
    stock_inicial_raw: Option<String>,
    unidad_medida: Option<String>,
    _codigos: Vec<(String, String)>,
    atributos: Vec<(String, Value)>,
}

fn parse_producto_row(linea: u32, value: &Value) -> Result<ProductoRow, SeedIssue> {
    let archivo = "productos.jsonl";
    let sku_raw = require_str(value, "sku", archivo, linea)?;
    if Sku::try_new(sku_raw).is_err() {
        return Err(SeedIssue::new(archivo, linea, "sku", "invalid sku"));
    }
    for field in ["nombre", "tipo", "categoria"] {
        let s = require_str(value, field, archivo, linea)?;
        if s.trim().is_empty() {
            return Err(SeedIssue::new(archivo, linea, field, "must not be empty"));
        }
    }
    let categoria = require_str(value, "categoria", archivo, linea)?.to_string();
    if Slug::parse(&categoria).is_err() {
        return Err(SeedIssue::new(
            archivo,
            linea,
            "categoria",
            "invalid slug format",
        ));
    }
    let marca = match value.get("marca") {
        None | Some(Value::Null) => None,
        Some(Value::String(s)) => {
            if Slug::parse(s).is_err() {
                return Err(SeedIssue::new(
                    archivo,
                    linea,
                    "marca",
                    "invalid slug format",
                ));
            }
            Some(s.clone())
        }
        Some(_) => {
            return Err(SeedIssue::new(
                archivo,
                linea,
                "marca",
                "must be a slug string or null",
            ));
        }
    };
    let precio_raw = match value.get("precio") {
        Some(Value::String(s)) => s.clone(),
        Some(_) => {
            return Err(SeedIssue::new(
                archivo,
                linea,
                "precio",
                "must be a decimal string",
            ));
        }
        None => {
            return Err(SeedIssue::new(archivo, linea, "precio", "required"));
        }
    };
    let stock_inicial_raw = match value.get("stock_inicial") {
        None => None,
        Some(Value::String(s)) => Some(s.clone()),
        Some(_) => {
            return Err(SeedIssue::new(
                archivo,
                linea,
                "stock_inicial",
                "must be a decimal string",
            ));
        }
    };
    let unidad_medida = match value.get("unidad_medida") {
        None => None,
        Some(Value::String(s)) => Some(s.clone()),
        Some(_) => {
            return Err(SeedIssue::new(
                archivo,
                linea,
                "unidad_medida",
                "must be a string",
            ));
        }
    };
    let mut codigos = Vec::new();
    if let Some(Value::Array(arr)) = value.get("codigos") {
        for (i, item) in arr.iter().enumerate() {
            let tipo = item.get("tipo").and_then(Value::as_str).ok_or_else(|| {
                SeedIssue::new(archivo, linea, format!("codigos[{i}].tipo"), "required")
            })?;
            if TipoCodigo::parse_wire(tipo).is_err() {
                return Err(SeedIssue::new(
                    archivo,
                    linea,
                    format!("codigos[{i}].tipo"),
                    "invalid tipo",
                ));
            }
            let valor = item.get("valor").and_then(Value::as_str).ok_or_else(|| {
                SeedIssue::new(archivo, linea, format!("codigos[{i}].valor"), "required")
            })?;
            if valor.trim().is_empty() {
                return Err(SeedIssue::new(
                    archivo,
                    linea,
                    format!("codigos[{i}].valor"),
                    "must not be empty",
                ));
            }
            codigos.push((tipo.to_string(), valor.to_string()));
        }
    }
    let mut atributos = Vec::new();
    if let Some(Value::Array(arr)) = value.get("atributos") {
        for (i, item) in arr.iter().enumerate() {
            let clave = item.get("clave").and_then(Value::as_str).ok_or_else(|| {
                SeedIssue::new(archivo, linea, format!("atributos[{i}].clave"), "required")
            })?;
            let valor = item.get("valor").cloned().unwrap_or(Value::Null);
            atributos.push((clave.to_string(), valor));
        }
    }
    Ok(ProductoRow {
        sku: sku_raw.to_string(),
        categoria,
        marca,
        precio_raw,
        stock_inicial_raw,
        unidad_medida,
        _codigos: codigos,
        atributos,
    })
}

fn validate_producto_record(
    linea: u32,
    row: &ProductoRow,
    categoria_slugs: &HashSet<String>,
    marca_slugs: &HashSet<String>,
    atributo_claves: &HashSet<String>,
) -> Vec<SeedIssue> {
    let archivo = "productos.jsonl";
    let mut issues = Vec::new();
    if !categoria_slugs.contains(&row.categoria) {
        issues.push(SeedIssue::new(
            archivo,
            linea,
            "categoria",
            "unknown category slug",
        ));
    }
    if let Some(marca) = &row.marca
        && !marca_slugs.contains(marca)
    {
        issues.push(SeedIssue::new(
            archivo,
            linea,
            "marca",
            "unknown brand slug",
        ));
    }
    if Decimal::from_str(&row.precio_raw).is_err() {
        issues.push(SeedIssue::new(archivo, linea, "precio", "invalid decimal"));
    } else if let Ok(d) = Decimal::from_str(&row.precio_raw)
        && d < Decimal::from_str("0").expect("zero")
    {
        issues.push(SeedIssue::new(
            archivo,
            linea,
            "precio",
            "must not be negative",
        ));
    }
    if let Some(stock) = &row.stock_inicial_raw
        && Decimal::from_str(stock).is_err()
    {
        issues.push(SeedIssue::new(
            archivo,
            linea,
            "stock_inicial",
            "invalid decimal",
        ));
    }
    if let Some(um) = &row.unidad_medida
        && um.parse::<UnidadMedida>().is_err()
    {
        issues.push(SeedIssue::new(
            archivo,
            linea,
            "unidad_medida",
            "invalid unidad_medida",
        ));
    }
    for (i, (clave, _)) in row.atributos.iter().enumerate() {
        if !atributo_claves.contains(clave) {
            issues.push(SeedIssue::new(
                archivo,
                linea,
                format!("atributos[{i}].clave"),
                "unknown atributo clave",
            ));
        }
    }
    issues
}

fn validate_relaciones(path: &Path, product_skus: &HashSet<String>) -> Vec<SeedIssue> {
    let archivo = "relaciones.jsonl";
    let mut issues = Vec::new();
    let mut rows: Vec<(u32, Value)> = Vec::new();
    issues.extend(read_jsonl(
        path,
        archivo,
        |_linea, value| Ok(value.clone()),
        &mut rows,
    ));
    for (linea, value) in rows {
        for field in ["origen", "destino", "tipo"] {
            if require_str(&value, field, archivo, linea).is_err() {
                issues.push(SeedIssue::new(archivo, linea, field, "required"));
            }
        }
        if let Ok(tipo) = require_str(&value, "tipo", archivo, linea)
            && !matches!(tipo, "similar_a" | "sustituible_por" | "complementa")
        {
            issues.push(SeedIssue::new(
                archivo,
                linea,
                "tipo",
                "invalid relation tipo",
            ));
        }
        if let Ok(origen) = require_str(&value, "origen", archivo, linea)
            && !product_skus.contains(origen)
        {
            issues.push(SeedIssue::new(archivo, linea, "origen", "unknown sku"));
        }
        if let Ok(destino) = require_str(&value, "destino", archivo, linea)
            && !product_skus.contains(destino)
        {
            issues.push(SeedIssue::new(archivo, linea, "destino", "unknown sku"));
        }
    }
    issues
}

fn validate_sinonimos(path: &Path) -> Vec<SeedIssue> {
    let archivo = "sinonimos.jsonl";
    let mut issues = Vec::new();
    let mut rows: Vec<(u32, Value)> = Vec::new();
    issues.extend(read_jsonl(
        path,
        archivo,
        |_linea, value| Ok(value.clone()),
        &mut rows,
    ));
    for (linea, value) in rows {
        let tipo = match require_str(&value, "tipo", archivo, linea) {
            Ok(t) => t,
            Err(e) => {
                issues.push(e);
                continue;
            }
        };
        if tipo != "mutuo" && tipo != "unidireccional" {
            issues.push(SeedIssue::new(
                archivo,
                linea,
                "tipo",
                "invalid sinonimo tipo",
            ));
            continue;
        }
        match value.get("equivalentes") {
            Some(Value::Array(arr)) if !arr.is_empty() && arr.iter().all(|v| v.is_string()) => {}
            _ => issues.push(SeedIssue::new(
                archivo,
                linea,
                "equivalentes",
                "required non-empty string array",
            )),
        }
        if tipo == "unidireccional" {
            match value.get("termino") {
                Some(Value::String(s)) if !s.trim().is_empty() => {}
                _ => issues.push(SeedIssue::new(
                    archivo,
                    linea,
                    "termino",
                    "required for unidireccional",
                )),
            }
        }
    }
    issues
}

fn validate_notas(path: &Path, product_skus: &HashSet<String>) -> Vec<SeedIssue> {
    let archivo = "notas.jsonl";
    let mut issues = Vec::new();
    let mut rows: Vec<(u32, Value)> = Vec::new();
    issues.extend(read_jsonl(
        path,
        archivo,
        |_linea, value| Ok(value.clone()),
        &mut rows,
    ));
    for (linea, value) in rows {
        for field in ["titulo", "contenido"] {
            if require_str(&value, field, archivo, linea).is_err() {
                issues.push(SeedIssue::new(archivo, linea, field, "required"));
            }
        }
        if let Some(Value::Array(skus)) = value.get("productos") {
            for (i, sku) in skus.iter().enumerate() {
                let Some(s) = sku.as_str() else {
                    issues.push(SeedIssue::new(
                        archivo,
                        linea,
                        format!("productos[{i}]"),
                        "must be sku string",
                    ));
                    continue;
                };
                if !product_skus.contains(s) {
                    issues.push(SeedIssue::new(
                        archivo,
                        linea,
                        format!("productos[{i}]"),
                        "unknown sku",
                    ));
                }
            }
        }
    }
    issues
}

fn validate_imagenes_dir(path: &Path, product_skus: &HashSet<String>) -> Vec<SeedIssue> {
    let mut issues = Vec::new();
    let entries = match fs::read_dir(path) {
        Ok(e) => e,
        Err(e) => {
            return vec![SeedIssue::new(
                "imagenes",
                0,
                "",
                format!("cannot read imagenes directory: {e}"),
            )];
        }
    };
    for entry in entries.flatten() {
        let file_type = match entry.file_type() {
            Ok(t) => t,
            Err(_) => continue,
        };
        if !file_type.is_dir() {
            continue;
        }
        let name = entry.file_name();
        let Some(sku) = name.to_str() else {
            continue;
        };
        if !product_skus.contains(sku) {
            issues.push(SeedIssue::new(
                "imagenes",
                0,
                sku,
                "directory name is not a product sku",
            ));
        }
    }
    issues
}

fn require_str<'a>(
    value: &'a Value,
    field: &str,
    archivo: &str,
    linea: u32,
) -> Result<&'a str, SeedIssue> {
    value
        .get(field)
        .and_then(Value::as_str)
        .ok_or_else(|| SeedIssue::new(archivo, linea, field, "required"))
}

use std::str::FromStr as _;

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn unique_temp_dir() -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time")
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("vectra-seed-test-{nanos}"));
        fs::create_dir_all(&dir).expect("temp dir");
        dir
    }

    fn repo_seed_dir() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../seed")
    }

    fn copy_dir_all(src: &Path, dst: &Path) -> std::io::Result<()> {
        fs::create_dir_all(dst)?;
        for entry in fs::read_dir(src)? {
            let entry = entry?;
            let path = entry.path();
            let dest = dst.join(entry.file_name());
            if path.is_dir() {
                copy_dir_all(&path, &dest)?;
            } else {
                fs::copy(&path, &dest)?;
            }
        }
        Ok(())
    }

    fn read_jsonl_values(path: &Path) -> Vec<Value> {
        let content = fs::read_to_string(path).unwrap_or_default();
        content
            .lines()
            .filter(|l| !l.trim().is_empty())
            .filter_map(|line| serde_json::from_str::<Value>(line).ok())
            .collect()
    }

    fn product_skus_and_lines(seed_dir: &Path) -> Vec<(String, Value)> {
        let path = seed_dir.join("productos.jsonl");
        read_jsonl_values(&path)
            .into_iter()
            .filter_map(|v| {
                v.get("sku")
                    .and_then(Value::as_str)
                    .map(|s| (s.to_string(), v.clone()))
            })
            .collect()
    }

    fn write_minimal_seed(dir: &Path, productos_line: &str) {
        fs::write(
            dir.join("categorias.jsonl"),
            r#"{"slug":"fijaciones","nombre":"Fijaciones"}
{"slug":"pernos","nombre":"Pernos","padre":"fijaciones"}
"#,
        )
        .expect("write categorias");
        fs::write(
            dir.join("marcas.jsonl"),
            r#"{"slug":"fixser","nombre":"Fixser"}
"#,
        )
        .expect("write marcas");
        fs::write(
            dir.join("atributos.jsonl"),
            r#"{"clave":"medida","etiqueta":"Medida","tipo":"texto"}
"#,
        )
        .expect("write atributos");
        fs::write(dir.join("productos.jsonl"), productos_line).expect("write productos");
    }

    #[test]
    fn seed_valid_minimal_directory_has_no_issues() {
        let dir = unique_temp_dir();
        write_minimal_seed(
            &dir,
            r#"{"sku":"PER-M8X40-ZN","nombre":"Perno hexagonal M8 x 40 zincado","tipo":"Perno","categoria":"pernos","marca":"fixser","precio":"120.50","atributos":[{"clave":"medida","valor":"M8"}]}
"#,
        );
        let issues = validate_dir(&dir);
        assert!(issues.is_empty(), "issues: {issues:?}");
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn seed_rejects_non_numeric_precio_with_file_line_field() {
        let dir = unique_temp_dir();
        write_minimal_seed(
            &dir,
            r#"{"sku":"PER-M8X40-ZN","nombre":"Perno","tipo":"Perno","categoria":"pernos","marca":"fixser","precio":"barato"}
"#,
        );
        let issues = validate_dir(&dir);
        assert!(
            issues
                .iter()
                .any(|i| { i.archivo == "productos.jsonl" && i.linea == 1 && i.campo == "precio" }),
            "issues: {issues:?}"
        );
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn seed_rejects_malformed_jsonl_line() {
        let dir = unique_temp_dir();
        write_minimal_seed(
            &dir,
            r#"{"sku":"PER-M8X40-ZN","nombre":"Perno hexagonal M8 x 40 zincado","tipo":"Perno","categoria":"pernos","marca":"fixser","precio":"120.50","atributos":[{"clave":"medida","valor":"M8"}]}
"#,
        );
        fs::write(
            dir.join("categorias.jsonl"),
            "{\"slug\":\"fijaciones\",\"nombre\":\"Fijaciones\"}\nNOT JSON\n",
        )
        .expect("write bad json");
        let issues = validate_dir(&dir);
        assert!(
            issues
                .iter()
                .any(|i| { i.archivo == "categorias.jsonl" && i.linea == 2 && i.campo.is_empty() }),
            "issues: {issues:?}"
        );
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn seed_rejects_unknown_categoria_slug() {
        let dir = unique_temp_dir();
        write_minimal_seed(
            &dir,
            r#"{"sku":"PER-M8X40-ZN","nombre":"Perno","tipo":"Perno","categoria":"no-existe","marca":"fixser","precio":"120.50"}
"#,
        );
        let issues = validate_dir(&dir);
        assert!(
            issues
                .iter()
                .any(|i| { i.archivo == "productos.jsonl" && i.campo == "categoria" }),
            "issues: {issues:?}"
        );
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn seed_rejects_duplicate_sku() {
        let dir = unique_temp_dir();
        write_minimal_seed(
            &dir,
            r#"{"sku":"PER-M8X40-ZN","nombre":"Perno","tipo":"Perno","categoria":"pernos","precio":"120.50"}
{"sku":"PER-M8X40-ZN","nombre":"Otro","tipo":"Perno","categoria":"pernos","precio":"1.00"}
"#,
        );
        let issues = validate_dir(&dir);
        assert!(
            issues
                .iter()
                .any(|i| { i.archivo == "productos.jsonl" && i.linea == 2 && i.campo == "sku" }),
            "issues: {issues:?}"
        );
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn seed_rejects_unknown_atributo_clave() {
        let dir = unique_temp_dir();
        write_minimal_seed(
            &dir,
            r#"{"sku":"PER-M8X40-ZN","nombre":"Perno","tipo":"Perno","categoria":"pernos","precio":"120.50","atributos":[{"clave":"inexistente","valor":"x"}]}
"#,
        );
        let issues = validate_dir(&dir);
        assert!(
            issues
                .iter()
                .any(|i| { i.archivo == "productos.jsonl" && i.campo == "atributos[0].clave" }),
            "issues: {issues:?}"
        );
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn seed_repo_seed_dir_has_no_issues() {
        let seed_dir = repo_seed_dir();
        let issues = validate_dir(&seed_dir);
        assert!(
            issues.is_empty(),
            "repo seed/ issues (is {:?} present?): {issues:?}",
            seed_dir
        );
    }

    #[test]
    fn seed_dev_dataset_has_expected_sku_count() {
        let seed_dir = repo_seed_dir();
        let products = product_skus_and_lines(&seed_dir);
        let unique: std::collections::HashSet<_> =
            products.iter().map(|(s, _)| s.as_str()).collect();
        let count = unique.len();
        assert!(
            (180..=220).contains(&count),
            "expected 180–220 SKUs, got {count}"
        );
        assert!(unique.contains("PER-M8X40-ZN"));
    }

    #[test]
    fn seed_dev_dataset_has_rubro_attributes_oem_and_typo_fixtures() {
        let seed_dir = repo_seed_dir();
        let required_attrs = ["medida", "largo", "grado", "material", "rosca"];
        let defs: std::collections::HashSet<_> =
            read_jsonl_values(&seed_dir.join("atributos.jsonl"))
                .into_iter()
                .filter_map(|v| v.get("clave").and_then(Value::as_str).map(str::to_string))
                .collect();
        for key in required_attrs {
            assert!(defs.contains(key), "missing atributo_def {key}");
        }
        let products = product_skus_and_lines(&seed_dir);
        for key in required_attrs {
            let used = products.iter().any(|(_, v)| {
                v.get("atributos")
                    .and_then(Value::as_array)
                    .is_some_and(|arr| {
                        arr.iter()
                            .any(|a| a.get("clave").and_then(Value::as_str) == Some(key))
                    })
            });
            assert!(used, "no product uses atributo {key}");
        }

        let mut oem_counts: std::collections::HashMap<String, usize> =
            std::collections::HashMap::new();
        for (_, v) in &products {
            if let Some(codes) = v.get("codigos").and_then(Value::as_array) {
                for c in codes {
                    if c.get("tipo").and_then(Value::as_str) == Some("oem") {
                        if let Some(val) = c.get("valor").and_then(Value::as_str) {
                            *oem_counts.entry(val.to_string()).or_default() += 1;
                        }
                    }
                }
            }
        }
        assert!(
            oem_counts.values().any(|&n| n >= 2),
            "expected shared OEM across two SKUs, counts: {oem_counts:?}"
        );

        let nombre_has = |needle: &str| {
            products.iter().any(|(_, v)| {
                v.get("nombre")
                    .and_then(Value::as_str)
                    .is_some_and(|n| n.to_lowercase().contains(needle))
            })
        };
        assert!(nombre_has("tornillo"));
        assert!(nombre_has("arandela"));
        assert!(nombre_has("bujía") || nombre_has("bujia"));

        let nombre_exact = |want: &str| {
            products
                .iter()
                .any(|(_, v)| v.get("nombre").and_then(Value::as_str) == Some(want))
        };
        assert!(nombre_exact("Tornillo autoperforante 10 x 1"));
        assert!(nombre_exact("Perno de Acero"));
        assert!(nombre_exact("Arandela para Perno"));

        let stock = |want: &str| {
            products
                .iter()
                .any(|(_, v)| v.get("stock_inicial").and_then(Value::as_str) == Some(want))
        };
        assert!(stock("40"));
        assert!(stock("3"));
    }

    #[test]
    fn seed_dev_dataset_has_relations_synonyms_notes_and_example_images() {
        let seed_dir = repo_seed_dir();
        let rel_path = seed_dir.join("relaciones.jsonl");
        assert!(rel_path.is_file(), "missing relaciones.jsonl");
        let rels = read_jsonl_values(&rel_path);
        for tipo in ["similar_a", "sustituible_por", "complementa"] {
            assert!(
                rels.iter()
                    .any(|r| r.get("tipo").and_then(Value::as_str) == Some(tipo)),
                "missing relacion tipo {tipo}"
            );
        }

        let sin_path = seed_dir.join("sinonimos.jsonl");
        assert!(sin_path.is_file());
        let sins = read_jsonl_values(&sin_path);
        assert!(
            sins.iter()
                .any(|s| s.get("tipo").and_then(Value::as_str) == Some("mutuo"))
        );
        assert!(
            sins.iter()
                .any(|s| s.get("tipo").and_then(Value::as_str) == Some("unidireccional"))
        );

        let notas_path = seed_dir.join("notas.jsonl");
        assert!(notas_path.is_file());
        let notas = read_jsonl_values(&notas_path);
        assert!(
            (2..=3).contains(&notas.len()),
            "expected 2–3 notas, got {}",
            notas.len()
        );

        let imagenes = seed_dir.join("imagenes");
        assert!(imagenes.is_dir());
        let skus: std::collections::HashSet<_> = product_skus_and_lines(&seed_dir)
            .into_iter()
            .map(|(s, _)| s)
            .collect();
        let mut found_image = false;
        for entry in fs::read_dir(&imagenes).expect("read imagenes") {
            let entry = entry.expect("entry");
            if !entry.file_type().expect("type").is_dir() {
                continue;
            }
            let name = entry.file_name();
            let sku = name.to_str().expect("sku dir");
            assert!(skus.contains(sku), "imagen dir {sku} is not a product sku");
            for file in fs::read_dir(entry.path()).expect("read sku dir") {
                let file = file.expect("file");
                let fname = file.file_name();
                let s = fname.to_string_lossy();
                if s.ends_with(".png")
                    || s.ends_with(".jpg")
                    || s.ends_with(".jpeg")
                    || s.ends_with(".webp")
                {
                    found_image = true;
                }
            }
        }
        assert!(
            found_image,
            "expected at least one image file under imagenes/"
        );
    }

    #[test]
    fn seed_dev_copy_rejects_non_numeric_precio() {
        let src = repo_seed_dir();
        let dst = unique_temp_dir();
        copy_dir_all(&src, &dst).expect("copy seed");
        let prod_path = dst.join("productos.jsonl");
        let mut content = fs::read_to_string(&prod_path).expect("read productos");
        if !content.is_empty() && !content.ends_with('\n') {
            content.push('\n');
        }
        content.push_str(
            r#"{"sku":"BAD-PRECIO","nombre":"X","tipo":"X","categoria":"pernos","precio":"barato"}
"#,
        );
        fs::write(&prod_path, content).expect("append bad precio");
        let issues = validate_dir(&dst);
        assert!(
            issues
                .iter()
                .any(|i| i.archivo == "productos.jsonl" && i.campo == "precio"),
            "issues: {issues:?}"
        );
        assert!(validate_dir(&src).is_empty());
        let _ = fs::remove_dir_all(&dst);
    }

    #[test]
    fn seed_dev_copy_rejects_unknown_relacion_sku() {
        let src = repo_seed_dir();
        let dst = unique_temp_dir();
        copy_dir_all(&src, &dst).expect("copy seed");
        let rel_path = dst.join("relaciones.jsonl");
        let mut content = fs::read_to_string(&rel_path).unwrap_or_default();
        if !content.is_empty() && !content.ends_with('\n') {
            content.push('\n');
        }
        content.push_str(
            r#"{"origen":"PER-M8X40-ZN","destino":"SKU-INEXISTENTE","tipo":"similar_a"}
"#,
        );
        fs::write(&rel_path, content).expect("append bad relacion");
        let issues = validate_dir(&dst);
        assert!(
            issues
                .iter()
                .any(|i| i.archivo == "relaciones.jsonl" && i.campo == "destino"),
            "issues: {issues:?}"
        );
        assert!(validate_dir(&src).is_empty());
        let _ = fs::remove_dir_all(&dst);
    }

    #[test]
    fn seed_dev_copy_rejects_unknown_imagen_sku() {
        let src = repo_seed_dir();
        let dst = unique_temp_dir();
        copy_dir_all(&src, &dst).expect("copy seed");
        let bad = dst.join("imagenes/SKU-INEXISTENTE");
        fs::create_dir_all(&bad).expect("bad imagen dir");
        let issues = validate_dir(&dst);
        assert!(
            issues
                .iter()
                .any(|i| i.archivo == "imagenes" && i.campo == "SKU-INEXISTENTE"),
            "issues: {issues:?}"
        );
        assert!(validate_dir(&src).is_empty());
        let _ = fs::remove_dir_all(&dst);
    }
}
