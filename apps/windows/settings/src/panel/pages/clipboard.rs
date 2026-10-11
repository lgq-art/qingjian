//! 「剪贴板」页：局域网共享剪贴板开关（`[clipboard]`）、设备名称、密钥、端口与自动同步。

use windows_reactor::*;

use crate::panel::controls::{field, note, page};
use crate::panel::{Message, Settings};

pub(crate) fn view(settings: &Settings, context: &mut ViewContext<Settings>) -> View {
    let cb = &settings.config.clipboard;

    let rows = [
        field(
            "局域网共享剪贴板",
            "在同一 Wi-Fi 或局域网内的多台电脑与设备间实时同步剪贴板。数据经 ChaCha20-Poly1305 端到端加密，安全防窃听。",
            ToggleSwitch::new()
                .is_on(cb.enabled)
                .on_toggled(context.callback(Message::ClipboardEnabled)),
        ),
        field(
            "自动写入系统剪贴板",
            "开启后，从远端收到的文字直接更新至本机操作系统剪贴板，可随时用 Ctrl+V 粘贴。关掉则仅存入输入法内部历史。",
            ToggleSwitch::new()
                .is_on(cb.auto_sync)
                .on_toggled(context.callback(Message::ClipboardAutoSync)),
        ),
        field(
            "本机设备名称",
            "在局域网中展示给其他设备的友好名称。留空时自动使用系统计算机名。",
            TextBox::new()
                .text(&cb.device_name)
                .placeholder_text(&cb.effective_device_name())
                .on_text_changed(context.callback(Message::ClipboardDeviceName)),
        ),
        field(
            "同步配对密钥",
            "同一局域网内的所有设备必须配置相同的密钥才能建立连接与解密内容。若不同则无法互通。",
            PasswordBox::new()
                .password(&cb.passphrase)
                .on_password_changed(context.callback(Message::ClipboardPassphrase)),
        ),
        field(
            "通信与发现端口",
            "UDP 发现广播与 TCP 数据流通信端口（默认 41258，范围 1024–65535）。",
            NumberBox::new()
                .value(cb.port as f64)
                .minimum(1024.0)
                .maximum(65535.0)
                .spin_button_placement_mode(NumberBoxSpinButtonPlacementMode::Inline)
                .on_value_changed(context.callback(Message::ClipboardPort)),
        ),
    ];

    let content = StackPanel::new().spacing(16.0).children([
        StackPanel::new().spacing(12.0).children(rows).into(),
        note("提示：局域网共享功能需要 Windows 防火墙允许青简服务端（qingjian-server.exe）访问专用网络的 UDP 与 TCP 端口。").into(),
    ]);

    page("剪贴板", content)
}
