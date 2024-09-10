#![allow(unused_imports)]

use std::time::Duration;
use std::thread::sleep;
use std::ffi::OsStr;

use quill::template as Template;
use quill::xml as Xml;
use quill::selector as Select;

pub fn main() {
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

    let index_dom = dom_store.append("index".into(), index_template).unwrap();

    {
        let dom_guard = index_dom.read().unwrap();
        println!("Found {} root elements!", dom_guard.nodes.len());
        let first = (**dom_guard.nodes.get(0).unwrap()).read().unwrap().name.clone();
        println!("First element is of type '{}'", first);
    }

    let row_selector = Select::NodeSelector::new()
        .named("Container".into())
            .child()
            .named("Row".into())
        .lock()
        ;
    println!("Searching for {}", row_selector.read().unwrap().to_string());

    // loop for profiling purposes
    loop {
        template_store.has("index".into());
        dom_store.has("index".into());
        
        let mut selector_handle = row_selector.write().unwrap();

        let mut row_count: u32 = 0;
        for root in index_dom.read().unwrap().nodes.iter() {
            for _ in selector_handle.apply(root.clone()).iter() {
                row_count += 1;
            }
        }
        println!("There are {} 'Container > Row' elements!", row_count);

        sleep(Duration::from_secs(1));
    }

}
