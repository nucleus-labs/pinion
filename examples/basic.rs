
use std::ffi::OsStr;

use rs_quill::template as Template;
use rs_quill::xml as Xml;
// use rs_quill::selector as Select;

pub fn main() {
    // Initialize the dom
    let template_store = Template::Store::new();

    let index_template = template_store.append("index".into(), OsStr::new("static/xml/mrpacker.xml".into())).unwrap();
    template_store.append("home".into(), OsStr::new("static/xml/home.xml".into())).unwrap();

    {
        let mut env_guard = template_store.env.write().unwrap();

        env_guard.add_global("title", "Mr. Packer V0.1.0");
        
        env_guard.add_filter("make_nav_button", |nav_page_name: &str| -> String {
            format!("<button class=\"nav\" id=\"{}\" />", nav_page_name)
        });
    
        env_guard.add_filter("make_special_button", |button_id: &str| -> String {
            format!("<button class=\"special\" id=\"{}\" />", button_id)
        });

        
        let store_clone = template_store.clone();
        env_guard.add_function("include_tree", move |index: String| -> String {
            store_clone
                .get(index)
                .read()
                .unwrap()
                .source
                .clone()
        });
    }

    let dom_store = Xml::Store::new();

    let _index_dom = dom_store.append("index".into(), index_template).unwrap();

    // let row_selector = Select::NodeSelector::new()
    //     .named("Row")
    //     ;

}
