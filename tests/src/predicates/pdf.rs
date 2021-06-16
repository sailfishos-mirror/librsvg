extern crate chrono;
extern crate lopdf;

use chrono::{DateTime, Utc};
use predicates::prelude::*;
use predicates::reflection::{Case, Child, PredicateReflection, Product};
use std::cmp;
use std::fmt;

/// Checks that the variable of type [u8] can be parsed as a PDF file.
#[derive(Debug)]
pub struct PdfPredicate {}

impl PdfPredicate {
    pub fn with_page_count(self: Self, num_pages: usize) -> DetailPredicate<Self> {
        DetailPredicate::<Self> {
            p: self,
            d: Detail::PageCount(num_pages),
        }
    }

    pub fn with_page_size(
        self: Self,
        width_in_points: f64,
        height_in_points: f64,
    ) -> DetailPredicate<Self> {
        DetailPredicate::<Self> {
            p: self,
            d: Detail::PageSize(Dimensions {
                w: width_in_points,
                h: height_in_points,
                unit: 1.0,
            }),
        }
    }

    pub fn with_creation_date(self: Self, when: DateTime<Utc>) -> DetailPredicate<Self> {
        DetailPredicate::<Self> {
            p: self,
            d: Detail::CreationDate(when),
        }
    }
}

impl Predicate<[u8]> for PdfPredicate {
    fn eval(&self, data: &[u8]) -> bool {
        lopdf::Document::load_mem(data).is_ok()
    }

    fn find_case<'a>(&'a self, _expected: bool, data: &[u8]) -> Option<Case<'a>> {
        match lopdf::Document::load_mem(data) {
            Ok(_) => None,
            Err(e) => Some(Case::new(Some(self), false).add_product(Product::new("Error", e))),
        }
    }
}

impl PredicateReflection for PdfPredicate {}

impl fmt::Display for PdfPredicate {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "is a PDF")
    }
}

/// Extends a PdfPredicate by a check for page count, page size or creation date.
#[derive(Debug)]
pub struct DetailPredicate<PdfPredicate> {
    p: PdfPredicate,
    d: Detail,
}

#[derive(Debug)]
enum Detail {
    PageCount(usize),
    PageSize(Dimensions),
    CreationDate(DateTime<Utc>),
}

/// A PDF page's dimensions from its `MediaBox`.
///
/// Note that `w` and `h` given in `UserUnit`, which is by default 1.0 = 1/72 inch.
#[derive(Debug)]
struct Dimensions {
    w: f64,
    h: f64,
    unit: f64, // UserUnit, in points (1/72 of an inch)
}

impl Dimensions {
    pub fn from_media_box(obj: &lopdf::Object, unit: Option<f64>) -> lopdf::Result<Dimensions> {
        let a = obj.as_array()?;
        Ok(Dimensions {
            w: a[2].as_float()?,
            h: a[3].as_float()?,
            unit: unit.unwrap_or(1.0),
        })
    }

    pub fn width_in_pt(self: &Self) -> f64 {
        self.w * self.unit
    }

    pub fn height_in_pt(self: &Self) -> f64 {
        self.h * self.unit
    }
}

impl fmt::Display for Dimensions {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} pt x {} pt", self.width_in_pt(), self.height_in_pt())
    }
}

impl cmp::PartialEq for Dimensions {
    fn eq(&self, other: &Self) -> bool {
        approx_eq!(
            f64,
            self.width_in_pt(),
            other.width_in_pt(),
            epsilon = 0.000_001
        ) && approx_eq!(
            f64,
            self.height_in_pt(),
            other.height_in_pt(),
            epsilon = 0.000_001
        )
    }
}

impl cmp::Eq for Dimensions {}

trait Details {
    fn get_page_count(&self) -> usize;
    fn get_page_size(&self) -> Option<Dimensions>;
    fn get_creation_date(&self) -> Option<DateTime<Utc>>;
    fn get_from_trailer<'a>(self: &'a Self, key: &[u8]) -> lopdf::Result<&'a lopdf::Object>;
    fn get_from_first_page<'a>(self: &'a Self, key: &[u8]) -> lopdf::Result<&'a lopdf::Object>;
}

impl DetailPredicate<PdfPredicate> {
    fn eval_doc(&self, doc: &lopdf::Document) -> bool {
        match &self.d {
            Detail::PageCount(n) => doc.get_page_count() == *n,
            Detail::PageSize(d) => doc.get_page_size().map_or(false, |dim| dim == *d),
            Detail::CreationDate(d) => doc.get_creation_date().map_or(false, |date| date == *d),
        }
    }

    fn find_case_for_doc<'a>(&'a self, expected: bool, doc: &lopdf::Document) -> Option<Case<'a>> {
        if self.eval_doc(doc) == expected {
            let product = self.product_for_doc(doc);
            Some(Case::new(Some(self), false).add_product(product))
        } else {
            None
        }
    }

    fn product_for_doc(&self, doc: &lopdf::Document) -> Product {
        match &self.d {
            Detail::PageCount(_) => Product::new(
                "actual page count",
                format!("{} page(s)", doc.get_page_count()),
            ),
            Detail::PageSize(_) => Product::new(
                "actual page size",
                match doc.get_page_size() {
                    Some(dim) => format!("{}", dim),
                    None => "None".to_string(),
                },
            ),
            Detail::CreationDate(_) => Product::new(
                "actual creation date",
                format!("{:?}", doc.get_creation_date()),
            ),
        }
    }
}

// Extensions to lopdf::Object; can be removed after lopdf 0.26
trait ObjExt {
    /// Get the object value as a float.
    /// Unlike as_f64() this will also cast an Integer to a Real.
    fn as_float(&self) -> lopdf::Result<f64>;
}

impl ObjExt for lopdf::Object {
    fn as_float(&self) -> lopdf::Result<f64> {
        match *self {
            lopdf::Object::Integer(ref value) => Ok(*value as f64),
            lopdf::Object::Real(ref value) => Ok(*value),
            _ => Err(lopdf::Error::Type),
        }
    }
}

impl Details for lopdf::Document {
    fn get_page_count(self: &Self) -> usize {
        self.get_pages().len()
    }

    fn get_page_size(self: &Self) -> Option<Dimensions> {
        match self.get_from_first_page(b"MediaBox") {
            Ok(obj) => {
                let unit = self
                    .get_from_first_page(b"UserUnit")
                    .and_then(ObjExt::as_float)
                    .ok();
                Dimensions::from_media_box(obj, unit).ok()
            }
            Err(_) => None,
        }
    }

    fn get_creation_date(self: &Self) -> Option<DateTime<Utc>> {
        match self.get_from_trailer(b"CreationDate") {
            Ok(obj) => obj.as_datetime().map(|date| date.with_timezone(&Utc)),
            Err(_) => None,
        }
    }

    fn get_from_trailer<'a>(self: &'a Self, key: &[u8]) -> lopdf::Result<&'a lopdf::Object> {
        let id = self.trailer.get(b"Info")?.as_reference()?;
        self.get_object(id)?.as_dict()?.get(key)
    }

    fn get_from_first_page<'a>(self: &'a Self, key: &[u8]) -> lopdf::Result<&'a lopdf::Object> {
        match self.page_iter().next() {
            Some(id) => self.get_object(id)?.as_dict()?.get(key),
            None => Err(lopdf::Error::ObjectNotFound),
        }
    }
}

impl Predicate<[u8]> for DetailPredicate<PdfPredicate> {
    fn eval(&self, data: &[u8]) -> bool {
        match lopdf::Document::load_mem(data) {
            Ok(doc) => self.eval_doc(&doc),
            _ => false,
        }
    }

    fn find_case<'a>(&'a self, expected: bool, data: &[u8]) -> Option<Case<'a>> {
        match lopdf::Document::load_mem(data) {
            Ok(doc) => self.find_case_for_doc(expected, &doc),
            Err(e) => Some(Case::new(Some(self), false).add_product(Product::new("Error", e))),
        }
    }
}

impl PredicateReflection for DetailPredicate<PdfPredicate> {
    fn children<'a>(&'a self) -> Box<dyn Iterator<Item = Child<'a>> + 'a> {
        let params = vec![Child::new("predicate", &self.p)];
        Box::new(params.into_iter())
    }
}

impl fmt::Display for DetailPredicate<PdfPredicate> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self.d {
            Detail::PageCount(n) => write!(f, "is a PDF with {} page(s)", n),
            Detail::PageSize(d) => write!(f, "is a PDF sized {}", d),
            Detail::CreationDate(d) => write!(f, "is a PDF created {:?}", d),
        }
    }
}
