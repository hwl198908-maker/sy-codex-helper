use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FeedbackRequest {
    pub endpoint_url: String,
    pub category: String,
    pub contact: String,
    pub message: String,
    pub app_version: String,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FeedbackResponse {
    pub ok: bool,
    pub message: String,
}

#[tauri::command]
pub fn submit_feedback(feedback: FeedbackRequest) -> Result<FeedbackResponse, String> {
    let endpoint = feedback.endpoint_url.trim();
    let parsed = reqwest::Url::parse(endpoint)
        .map_err(|_| "反馈地址无效，请检查服务器地址。".to_string())?;
    if !matches!(parsed.scheme(), "http" | "https") || parsed.host_str().is_none() {
        return Err("反馈地址无效，请使用完整的 http(s) 地址。".to_string());
    }
    if feedback.message.trim().len() < 6 {
        return Err("请填写更具体的反馈内容。".to_string());
    }

    let payload = serde_json::json!({
        "category": feedback.category.trim(),
        "contact": feedback.contact.trim(),
        "message": feedback.message.trim(),
        "appVersion": feedback.app_version.trim(),
        "os": std::env::consts::OS,
        "arch": std::env::consts::ARCH,
        "diagnosticLogPath": crate::diagnostics::log_path().to_string_lossy(),
    });

    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(12))
        .build()
        .map_err(|err| format!("创建反馈请求失败：{err}"))?;
    let response = client
        .post(parsed)
        .json(&payload)
        .send()
        .map_err(|err| format!("无法连接反馈服务器：{err}"))?;

    if !response.status().is_success() {
        return Err(format!("反馈服务器返回异常状态：{}", response.status()));
    }

    Ok(FeedbackResponse {
        ok: true,
        message: "反馈已提交，感谢你的建议。".to_string(),
    })
}
