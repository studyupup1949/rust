# Y Accordion RS Yew Usage

Adding Accordion RS to your project is simple:

1. Make sure your project is set up with **Yew**. Refer to their [Getting Started Guide](https://yew.rs/docs/getting-started/introduction) for setup instructions.

1. Add the Accordion component to your dependencies by including it in your `Cargo.toml` file.

   ```sh
   cargo add accordion-rs --features=yew
   ```

1. Import the `Accordion` component into your Yew component and start using it in your app.

## 🛠️ Usage

Incorporating Yew Accordion into your application is easy. Follow these steps:

1. Import the Accordion component into your Yew project:

   ```rust
   use yew::prelude::*;
   use accordion_rs::yew::Accordion;
   ```

1. Define the accordion sections and use the Accordion component in your Yew component:

   ```rust
   use accordion_rs::yew::{Accordion, Item, List};
   use accordion_rs::{Size, Align};
   use yew::prelude::*;

   #[function_component(App)]
   pub fn app() -> Html {
       let on_will_open = Callback::from(|_| log::info!("Accordion will open!"));
       let on_did_open = Callback::from(|_| log::info!("Accordion did open!"));
       let on_will_close = Callback::from(|_| log::info!("Accordion will close!"));
       let on_did_close = Callback::from(|_| log::info!("Accordion did close!"));
       html! {
           <Accordion
               size={Size::XLarge}
               expanded={html! { <h3>{ "Accordion Expanded" }</h3> }}
               collapsed={html! { <h3>{ "Accordion Collapsed" }</h3> }}
               class="bg-gray-900 text-gray-300 border border-gray-700 p-4 rounded-lg"
               expanded_class="text-white bg-gray-800"
               collapsed_class="text-gray-400 bg-gray-800"
               did_open={on_did_open.clone()}
               will_open={on_will_open.clone()}
               did_close={on_did_close.clone()}
               will_close={on_will_close.clone()}
           >
               <List>
                   <Item align={Align::Left}>{ "Item 1 - Left" }</Item>
                   <Item align={Align::Right}>{ "Item 2 - Right" }</Item>
               </List>
           </Accordion>
       }
   }
   ```

## 🔧 Props

### Main Props

| Property    | Type                   | Description                                                                     | Default         |
| ----------- | ---------------------- | ------------------------------------------------------------------------------- | --------------- |
| `expand`    | `UseStateHandle<bool>` | State handle managing whether the accordion is initially expanded or collapsed. | `false`         |
| `expanded`  | `Html`                 | Content to display when the accordion is expanded.                              | `""`            |
| `collapsed` | `Html`                 | Content to display when the accordion is collapsed.                             | `""`            |
| `children`  | `Html`                 | Child elements displayed within the accordion container.                        | `""`            |
| `size`      | `Size`                 | Size of the accordion (`"small"`, `"medium"`, `"large"`, etc.).                 | `Size::XXLarge` |
| `duration`  | `u64`                  | Animation duration for expand/collapse transitions, in milliseconds.            | `600`           |

### Styling Props

```sh
+-----------------------------------------------------------+
|                  [Accordion Container]                    |  <-- `class` & `style`
|                                                           |
|   +-----------------------------------------------+       |  <-- `collapsed_class` & `collapsed_style`
|   |             [Collapsed Content]               |       |  <-- `collapsed`
|   +-----------------------------------------------+       |
|                                                           |
|   +-----------------------------------------------+       |  <-- `expanded_class` & `expanded_style`
|   |             [Expanded Content]                |       |  <-- `expanded`
|   +-----------------------------------------------+       |
|                                                           |
|   +-----------------------------------------------+       |  <-- `content_class` & `content_style`
|   |              [Accordion Content]              |       |  <-- `children`
|   +-----------------------------------------------+       |
|                                                           |
+-----------------------------------------------------------+
```

| Property          | Type           | Description                                       | Default |
| ----------------- | -------------- | ------------------------------------------------- | ------- |
| `class`           | `&'static str` | CSS class for the accordion container.            | `""`    |
| `expanded_class`  | `&'static str` | CSS class for the expanded content.               | `""`    |
| `collapsed_class` | `&'static str` | CSS class for the collapsed content.              | `""`    |
| `content_class`   | `&'static str` | CSS class for the content section.                | `""`    |
| `style`           | `&'static str` | Custom inline styles for the accordion container. | `""`    |
| `expanded_style`  | `&'static str` | Custom inline styles for the expanded element.    | `""`    |
| `collapsed_style` | `&'static str` | Custom inline styles for the collapsed element.   | `""`    |
| `content_style`   | `&'static str` | Custom inline styles for the accordion content.   | `""`    |

### Callback Props

| Property     | Type           | Description                                     | Default |
| ------------ | -------------- | ----------------------------------------------- | ------- |
| `will_open`  | `Callback<()>` | Callback triggered before the accordion opens.  | No-op   |
| `did_open`   | `Callback<()>` | Callback triggered after the accordion opens.   | No-op   |
| `will_close` | `Callback<()>` | Callback triggered before the accordion closes. | No-op   |
| `did_close`  | `Callback<()>` | Callback triggered after the accordion closes.  | No-op   |

### A11Y Props

| Property        | Type           | Description                                                          | Default |
| --------------- | -------------- | -------------------------------------------------------------------- | ------- |
| `aria_controls` | `&'static str` | ARIA controls attribute for accessibility.                           | `""`    |
| `aria_enabled`  | `bool`         | Whether ARIA attributes are enabled for screen reader accessibility. | `true`  |

## 💡 Notes

- Use the `expand` prop to control the open/close state of the accordion programmatically.
- The `size` prop can be customized to adjust the width of the accordion.
- Callback props like `will_open` and `did_close` provide hooks to perform actions during the accordion's lifecycle.
- Custom classes and styles allow for extensive customization of the accordion's appearance and behavior.
- ARIA attributes can be enabled/disabled using the `aria_enabled` prop for better screen reader compatibility.
