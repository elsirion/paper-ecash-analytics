use wasm_bindgen::prelude::*;

/// Start the html5-qrcode scanner on the given element ID.
/// Calls the provided closure with each decoded string.
pub fn start_qr_scanner(
    element_id: &str,
    on_decode: impl Fn(String) + 'static,
) -> Result<(), String> {
    let callback = Closure::wrap(Box::new(move |text: JsValue| {
        if let Some(s) = text.as_string() {
            on_decode(s);
        }
    }) as Box<dyn FnMut(JsValue)>);

    let scanner_key = format!("__qr_scanner_{}", element_id);
    let callback_key = format!("__qr_callback_{}", element_id);

    // Set the callback on window so JS can call it
    let window = web_sys::window().ok_or("No window")?;
    js_sys::Reflect::set(
        &window,
        &JsValue::from_str(&callback_key),
        callback.as_ref(),
    )
    .map_err(|_| "Failed to set callback")?;
    callback.forget();

    let code = format!(
        r#"(function() {{
    if (typeof Html5Qrcode === 'undefined') {{
        throw new Error('html5-qrcode library not loaded');
    }}
    var scanner = new Html5Qrcode("{elem_id}");
    window["{scanner_key}"] = scanner;
    scanner.start(
        {{ facingMode: "environment" }},
        {{ fps: 10, qrbox: {{ width: 250, height: 250 }} }},
        window["{callback_key}"],
        function(errorMessage) {{}}
    ).catch(function(err) {{
        console.error("QR scanner start failed:", err);
    }});
}})()"#,
        elem_id = element_id,
        scanner_key = scanner_key,
        callback_key = callback_key,
    );

    js_sys::eval(&code).map_err(|e| format!("Failed to start scanner: {:?}", e))?;

    Ok(())
}

/// Stop the html5-qrcode scanner and clean up.
pub fn stop_qr_scanner(element_id: &str) {
    let scanner_key = format!("__qr_scanner_{}", element_id);
    let callback_key = format!("__qr_callback_{}", element_id);

    let code = format!(
        r#"(function() {{
    var scanner = window["{scanner_key}"];
    if (scanner) {{
        scanner.stop().then(function() {{
            scanner.clear();
        }}).catch(function(err) {{
            console.warn("QR scanner stop error:", err);
        }});
        delete window["{scanner_key}"];
        delete window["{callback_key}"];
    }}
}})()"#,
        scanner_key = scanner_key,
        callback_key = callback_key,
    );

    let _ = js_sys::eval(&code);
}
