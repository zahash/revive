
fn custom_main() {
    let r = Reactive::new(0);
    let s = r.derive(|val| val.to_string());

    let ui = Group {
        components: vec![
            Box::new(DynText { text: s }),
            Box::new(Button {
                label: "+".into(),
                onclick: Box::new({
                    let r = r.clone();
                    move || r.update_inplace_unchecked(|val| *val += 1)
                }),
            }),
            Box::new(Button {
                label: "-".into(),
                onclick: Box::new({
                    let r = r.clone();
                    move || r.update_inplace_unchecked(|val| *val -= 1)
                }),
            }),
        ],
    };

    ui.render(&Layout, &Style);
    r.set(10);
    ui.render(&Layout, &Style);
}

struct Layout;
struct Style;

trait Render {
    fn render(&self, layout: &Layout, style: &Style);
}

struct Group {
    components: Vec<Box<dyn Render>>,
}

impl Render for Group {
    fn render(&self, layout: &Layout, style: &Style) {
        for component in &self.components {
            component.render(layout, style);
        }
    }
}

struct Button {
    label: String,
    onclick: Box<dyn FnMut()>,
}

impl Render for Button {
    fn render(&self, layout: &Layout, style: &Style) {
        // use wasm_bindgen::prelude::Closure;
        // use wasm_bindgen::JsCast;

        // let a = self.onclick.deref();

        // let cb = Closure::wrap(
        //     Box::new(|_| self.onclick.deref_mut()()) as Box<dyn FnMut(web_sys::Event)>
        // );

        // let button = create_element("button");

        // body()
        //     .add_event_listener_with_callback("click", cb.as_ref().unchecked_ref())
        //     .unwrap();
        // cb.forget();

        println!("<Button label={} />", self.label);
    }
}

struct Text {
    text: String,
}

impl Render for Text {
    fn render(&self, layout: &Layout, style: &Style) {
        let node = document().create_text_node(&self.text);
        append_child(body(), &node);
    }
}

struct DynText {
    text: Reactive<String>,
}

impl Render for DynText {
    fn render(&self, layout: &Layout, style: &Style) {
        let node = document().create_text_node(&self.text.value());
        append_child(body(), &node);
        self.text.add_observer(move |val| node.set_data(val.as_ref()));
    }
}

#[inline]
fn window() -> web_sys::Window {
    web_sys::window().expect("cannot access the window object")
}

#[inline]
fn document() -> web_sys::Document {
    window()
        .document()
        .expect("cannot access the document object")
}

#[inline]
fn body() -> web_sys::HtmlElement {
    document().body().expect("cannot access the body tag")
}

#[inline]
fn append_child(element: web_sys::HtmlElement, node: &web_sys::Node) -> web_sys::Node {
    element
        .append_child(node)
        .expect(&format!("cannot append_child {:?} to {:?}", node, element))
}

#[inline]
fn create_element(tag: &str) -> web_sys::Element {
    document()
        .create_element(tag)
        .expect(&format!("cannot create_element {}", tag))
}
