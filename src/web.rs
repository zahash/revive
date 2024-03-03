fn web_sys_main() {
    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();
    let body = document.body().unwrap();

    let count = Reactive::new(0);

    let root = El::new("div")
        .text("count: ")
        .dyn_text(count.derive(|val| val.to_string()))
        .child(button("-1", {
            let count = count.clone();
            move |_| count.update(|n| n - 1)
        }))
        .child(button("+1", move |_| count.update(|n| n + 1)));

    body.append_child(&root).unwrap();
}

fn button(label: &str, cb: impl FnMut(web_sys::Event) + 'static) -> El {
    El::new("button").on("click", cb).text(label)
}

#[derive(Debug, Clone)]
pub struct El(web_sys::Element);

impl std::ops::Deref for El {
    type Target = web_sys::Element;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl El {
    pub fn new(tag_name: &str) -> Self {
        let window = web_sys::window().unwrap();
        let document = window.document().unwrap();
        let el = document.create_element(tag_name).unwrap();
        Self(el)
    }

    pub fn on(self, event_name: &str, cb: impl FnMut(web_sys::Event) + 'static) -> Self {
        use wasm_bindgen::prelude::Closure;
        use wasm_bindgen::JsCast;
        let cb = Closure::wrap(Box::new(cb) as Box<dyn FnMut(web_sys::Event)>);
        self.0
            .add_event_listener_with_callback(event_name, cb.as_ref().unchecked_ref())
            .unwrap();
        cb.forget();
        self
    }

    pub fn attr(self, attr_name: &str, value: &str) -> Self {
        self.0.set_attribute(attr_name, value).unwrap();
        self
    }

    pub fn text(self, data: &str) -> Self {
        let window = web_sys::window().unwrap();
        let document = window.document().unwrap();
        let node = document.create_text_node(data);
        self.0.append_child(&node).unwrap();
        self
    }

    pub fn child(self, child: El) -> Self {
        self.0.append_child(&child).unwrap();
        self
    }

    pub fn dyn_text<T: AsRef<str>>(self, r: Reactive<T>) -> Self {
        let window = web_sys::window().unwrap();
        let document = window.document().unwrap();
        let node = document.create_text_node("");

        self.0.append_child(&node).unwrap();

        r.add_observer(move |val| node.set_data(val.as_ref()));

        self
    }
}