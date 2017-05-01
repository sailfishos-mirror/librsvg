// This file was generated by gir (2184662) from gir-files (ec4c204)
// DO NOT EDIT

use DimensionData;
use HandleFlags;
use PositionData;
use ffi;
use glib::Value;
use glib::translate::*;
use gobject_ffi;
use std::mem::transmute;

glib_wrapper! {
    pub struct Handle(Object<ffi::RsvgHandle>);

    match fn {
        get_type => || ffi::rsvg_handle_get_type(),
    }
}

impl Handle {
    pub fn new() -> Handle {
        unsafe {
            from_glib_full(ffi::rsvg_handle_new())
        }
    }

    //pub fn new_from_data(data: /*Unimplemented*/&CArray TypeId { ns_id: 0, id: 3 }, data_len: /*Unimplemented*/Fundamental: Size, error: /*Ignored*/Option<Error>) -> Handle {
    //    unsafe { TODO: call ffi::rsvg_handle_new_from_data() }
    //}

    //pub fn new_from_file(file_name: &str, error: /*Ignored*/Option<Error>) -> Handle {
    //    unsafe { TODO: call ffi::rsvg_handle_new_from_file() }
    //}

    //pub fn new_from_gfile_sync<'a, P: IsA</*Ignored*/gio::File>, Q: Into<Option<&'a /*Ignored*/gio::Cancellable>>>(file: &P, flags: HandleFlags, cancellable: Q, error: /*Ignored*/Option<Error>) -> Handle {
    //    unsafe { TODO: call ffi::rsvg_handle_new_from_gfile_sync() }
    //}

    //pub fn new_from_stream_sync<'a, 'b, P: IsA</*Ignored*/gio::InputStream>, Q: IsA</*Ignored*/gio::File> + 'a, R: Into<Option<&'a Q>>, S: Into<Option<&'b /*Ignored*/gio::Cancellable>>>(input_stream: &P, base_file: R, flags: HandleFlags, cancellable: S, error: /*Ignored*/Option<Error>) -> Handle {
    //    unsafe { TODO: call ffi::rsvg_handle_new_from_stream_sync() }
    //}

    pub fn new_with_flags(flags: HandleFlags) -> Handle {
        unsafe {
            from_glib_full(ffi::rsvg_handle_new_with_flags(flags.to_glib()))
        }
    }

    //pub fn close(&self, error: /*Ignored*/Option<Error>) -> bool {
    //    unsafe { TODO: call ffi::rsvg_handle_close() }
    //}

    pub fn get_base_uri(&self) -> Option<String> {
        unsafe {
            from_glib_none(ffi::rsvg_handle_get_base_uri(self.to_glib_none().0))
        }
    }

    pub fn get_dimensions(&self) -> DimensionData {
        unsafe {
            let mut dimension_data = DimensionData::uninitialized();
            ffi::rsvg_handle_get_dimensions(self.to_glib_none().0, dimension_data.to_glib_none_mut().0);
            dimension_data
        }
    }

    pub fn get_dimensions_sub<'a, P: Into<Option<&'a str>>>(&self, id: P) -> Option<DimensionData> {
        let id = id.into();
        let id = id.to_glib_none().0;
        unsafe {
            let mut dimension_data = DimensionData::uninitialized();
            let ret = from_glib(ffi::rsvg_handle_get_dimensions_sub(self.to_glib_none().0, dimension_data.to_glib_none_mut().0, id));
            if ret { Some(dimension_data) } else { None }
        }
    }

    //pub fn get_pixbuf(&self) -> /*Ignored*/Option<gdk_pixbuf::Pixbuf> {
    //    unsafe { TODO: call ffi::rsvg_handle_get_pixbuf() }
    //}

    //pub fn get_pixbuf_sub<'a, P: Into<Option<&'a str>>>(&self, id: P) -> /*Ignored*/Option<gdk_pixbuf::Pixbuf> {
    //    unsafe { TODO: call ffi::rsvg_handle_get_pixbuf_sub() }
    //}

    pub fn get_position_sub(&self, id: &str) -> Option<PositionData> {
        unsafe {
            let mut position_data = PositionData::uninitialized();
            let ret = from_glib(ffi::rsvg_handle_get_position_sub(self.to_glib_none().0, position_data.to_glib_none_mut().0, id.to_glib_none().0));
            if ret { Some(position_data) } else { None }
        }
    }

    pub fn has_sub(&self, id: &str) -> bool {
        unsafe {
            from_glib(ffi::rsvg_handle_has_sub(self.to_glib_none().0, id.to_glib_none().0))
        }
    }

    //pub fn read_stream_sync<'a, P: IsA</*Ignored*/gio::InputStream>, Q: Into<Option<&'a /*Ignored*/gio::Cancellable>>>(&self, stream: &P, cancellable: Q, error: /*Ignored*/Option<Error>) -> bool {
    //    unsafe { TODO: call ffi::rsvg_handle_read_stream_sync() }
    //}

    //pub fn render_cairo(&self, cr: /*Ignored*/&mut cairo::Context) -> bool {
    //    unsafe { TODO: call ffi::rsvg_handle_render_cairo() }
    //}

    //pub fn render_cairo_sub<'a, P: Into<Option<&'a str>>>(&self, cr: /*Ignored*/&mut cairo::Context, id: P) -> bool {
    //    unsafe { TODO: call ffi::rsvg_handle_render_cairo_sub() }
    //}

    //pub fn set_base_gfile<P: IsA</*Ignored*/gio::File>>(&self, base_file: &P) {
    //    unsafe { TODO: call ffi::rsvg_handle_set_base_gfile() }
    //}

    pub fn set_base_uri(&self, base_uri: &str) {
        unsafe {
            ffi::rsvg_handle_set_base_uri(self.to_glib_none().0, base_uri.to_glib_none().0);
        }
    }

    pub fn set_dpi(&self, dpi: f64) {
        unsafe {
            ffi::rsvg_handle_set_dpi(self.to_glib_none().0, dpi);
        }
    }

    pub fn set_dpi_x_y(&self, dpi_x: f64, dpi_y: f64) {
        unsafe {
            ffi::rsvg_handle_set_dpi_x_y(self.to_glib_none().0, dpi_x, dpi_y);
        }
    }

    //pub fn write(&self, buf: /*Unimplemented*/&CArray TypeId { ns_id: 0, id: 3 }, count: /*Unimplemented*/Fundamental: Size, error: /*Ignored*/Option<Error>) -> bool {
    //    unsafe { TODO: call ffi::rsvg_handle_write() }
    //}

    pub fn get_property_dpi_x(&self) -> f64 {
        let mut value = Value::from(&0f64);
        unsafe {
            gobject_ffi::g_object_get_property(self.to_glib_none().0, "dpi-x".to_glib_none().0, value.to_glib_none_mut().0);
        }
        value.get().unwrap()
    }

    pub fn set_property_dpi_x(&self, dpi_x: f64) {
        unsafe {
            gobject_ffi::g_object_set_property(self.to_glib_none().0, "dpi-x".to_glib_none().0, Value::from(&dpi_x).to_glib_none().0);
        }
    }

    pub fn get_property_dpi_y(&self) -> f64 {
        let mut value = Value::from(&0f64);
        unsafe {
            gobject_ffi::g_object_get_property(self.to_glib_none().0, "dpi-y".to_glib_none().0, value.to_glib_none_mut().0);
        }
        value.get().unwrap()
    }

    pub fn set_property_dpi_y(&self, dpi_y: f64) {
        unsafe {
            gobject_ffi::g_object_set_property(self.to_glib_none().0, "dpi-y".to_glib_none().0, Value::from(&dpi_y).to_glib_none().0);
        }
    }

    pub fn get_property_em(&self) -> f64 {
        let mut value = Value::from(&0f64);
        unsafe {
            gobject_ffi::g_object_get_property(self.to_glib_none().0, "em".to_glib_none().0, value.to_glib_none_mut().0);
        }
        value.get().unwrap()
    }

    pub fn get_property_ex(&self) -> f64 {
        let mut value = Value::from(&0f64);
        unsafe {
            gobject_ffi::g_object_get_property(self.to_glib_none().0, "ex".to_glib_none().0, value.to_glib_none_mut().0);
        }
        value.get().unwrap()
    }

    pub fn get_property_flags(&self) -> HandleFlags {
        let mut value = Value::from(&0u32);
        unsafe {
            gobject_ffi::g_object_get_property(self.to_glib_none().0, "flags".to_glib_none().0, value.to_glib_none_mut().0);
            from_glib(transmute(value.get::<u32>().unwrap()))
        }
    }

    pub fn get_property_height(&self) -> i32 {
        let mut value = Value::from(&0);
        unsafe {
            gobject_ffi::g_object_get_property(self.to_glib_none().0, "height".to_glib_none().0, value.to_glib_none_mut().0);
        }
        value.get().unwrap()
    }

    pub fn get_property_width(&self) -> i32 {
        let mut value = Value::from(&0);
        unsafe {
            gobject_ffi::g_object_get_property(self.to_glib_none().0, "width".to_glib_none().0, value.to_glib_none_mut().0);
        }
        value.get().unwrap()
    }
}
