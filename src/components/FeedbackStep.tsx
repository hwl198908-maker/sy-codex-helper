import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import {
  Alert,
  Button,
  Group,
  Paper,
  Select,
  Stack,
  Text,
  Textarea,
  TextInput,
  Title
} from "@mantine/core";
import { APP_VERSION, FEEDBACK_ENDPOINT_URL } from "../lib/defaults";

export function FeedbackStep() {
  const [category, setCategory] = useState("安装问题");
  const [contact, setContact] = useState("");
  const [message, setMessage] = useState("");
  const [status, setStatus] = useState("填写后点击提交，反馈会发送到后台。");
  const [isSubmitting, setIsSubmitting] = useState(false);

  async function submitFeedback() {
    setIsSubmitting(true);
    setStatus("正在提交反馈...");
    try {
      const result = await invoke<{ ok: boolean; message: string }>("submit_feedback", {
        feedback: {
          endpointUrl: FEEDBACK_ENDPOINT_URL,
          category,
          contact,
          message,
          appVersion: APP_VERSION,
        },
      });
      setStatus(result.message);
      if (result.ok) {
        setMessage("");
      }
    } catch (error) {
      setStatus(`提交失败：${error instanceof Error ? error.message : String(error)}`);
    } finally {
      setIsSubmitting(false);
    }
  }

  return (
    <Paper className="panel" radius="md" p="xl">
      <Stack gap="md">
        <div>
          <Text className="eyebrow">第 6 步</Text>
          <Title order={2}>意见反馈</Title>
          <Text c="dimmed" mt={6}>
            遇到安装、API、中文显示或更新问题，可以直接在这里提交。后台会保留版本和诊断信息，方便后续修复。
          </Text>
        </div>

        <Select
          label="问题类型"
          value={category}
          onChange={(value) => setCategory(value || "其他建议")}
          data={["安装问题", "API 配置", "Codex 中文显示", "版本更新", "OpenClaw 预留", "其他建议"]}
        />

        <TextInput
          label="联系方式"
          description="可填写微信、手机号或邮箱，方便需要时联系你。"
          value={contact}
          onChange={(event) => setContact(event.currentTarget.value)}
          placeholder="例如：weixxxnb"
        />

        <Textarea
          label="反馈内容"
          minRows={6}
          value={message}
          onChange={(event) => setMessage(event.currentTarget.value)}
          placeholder="请描述你点了哪里、看到什么提示、希望怎么改。"
        />

        <Alert color="gray" variant="light">
          当前版本：{APP_VERSION}。提交时会附带系统类型和诊断日志路径，不会上传 API Key。
        </Alert>

        <Group justify="space-between">
          <Text c="dimmed">{status}</Text>
          <Button className="primary-action" onClick={submitFeedback} loading={isSubmitting}>提交反馈</Button>
        </Group>
      </Stack>
    </Paper>
  );
}
