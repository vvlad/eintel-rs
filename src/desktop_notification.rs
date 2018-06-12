#[cfg(target_os = "macos")]
extern crate mac_notification_sys;

#[cfg(target_os = "linux")]
extern crate notify_rust;

#[cfg(target_os = "windows")]
extern crate winrt;

#[cfg(target_os = "macos")]
pub fn desktop_notification(msg_title: &str, msg_body: &str) {
    let bundle = mac_notification_sys::get_bundle_identifier("Script Editor").unwrap();
    mac_notification_sys::set_application(&bundle).unwrap();
    mac_notification_sys::send_notification(msg_title, &None, msg_body, &None).unwrap();
}

#[cfg(target_os = "linux")]
pub fn desktop_notification(msg_title: &str, msg_body: &str) {
    notify_rust::Notification::new()
        .icon("emblem-important")
        .summary(msg_title)
        .body(msg_body)
        .show()
        .unwrap();
}

#[cfg(target_os = "windows")]
pub fn desktop_notification(msg_title: &str, msg_body: &str) {
    use winrt::windows::data::xml::dom::*;
    use winrt::windows::ui::notifications::*;
    use winrt::*;
    unsafe {
        let toast_xml =
            ToastNotificationManager::get_template_content(ToastTemplateType_ToastText02).unwrap();
        let toast_text_elements = toast_xml
            .get_elements_by_tag_name(&FastHString::new("text"))
            .unwrap();

        toast_text_elements
            .item(0)
            .unwrap()
            .append_child(
                &*toast_xml
                    .create_text_node(&FastHString::from(msg_title))
                    .unwrap()
                    .query_interface::<IXmlNode>()
                    .unwrap(),
            )
            .unwrap();
        toast_text_elements
            .item(1)
            .unwrap()
            .append_child(
                &*toast_xml
                    .create_text_node(&FastHString::from(msg_body))
                    .unwrap()
                    .query_interface::<IXmlNode>()
                    .unwrap(),
            )
            .unwrap();

        let toast = ToastNotification::create_toast_notification(&*toast_xml).unwrap();
        ToastNotificationManager::create_toast_notifier_with_id(&FastHString::new(
            "{1AC14E77-02E7-4E5D-B744-2EB1AE5198B7}\\WindowsPowerShell\\v1.0\\powershell.exe",
        )).unwrap()
            .show(&*toast)
            .unwrap();
    }
}
