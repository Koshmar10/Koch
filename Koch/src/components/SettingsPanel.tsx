import { invoke } from "@tauri-apps/api/core"
import { useEffect, useState } from "react"
import { Settings } from "../../src-tauri/bindings/Settings"

export function SettingsPanel() {
    const [settings, setSettings] = useState<Settings | null>(null);
    useEffect(() => {
        let loadSettings = async () => {
            const s = await invoke<Settings>('get_settings');
            setSettings(s);
        }
        loadSettings();
    }, [])
    const handleInputUdate = (setting: string, val: string) => {
        invoke('update_settings', { key: setting, val: val })
        console.log(val);
    }

    return (
        <div className="text-3xl p-4">
            Setting Panel
            <div className="flex flex-col">
                <div>
                    <label className="mb-2">
                        chess.com user
                        <input
                            type="text"
                            className="block mt-1 p-1 border rounded text-[1.6rem] w-84 text-black"
                            placeholder="Enter your username"
                            value={settings?.map["chessdotcom_user"] ?? ""}
                            onChange={(e) => {
                                // Allow typing by updating local state
                                if (settings) {
                                    setSettings({
                                        ...settings,
                                        map: {
                                            ...settings.map,
                                            chessdotcom_user: e.target.value,
                                        },
                                    });
                                }
                            }}
                            onBlur={(e) => {
                                // Persist change on blur
                                handleInputUdate("chessdotcom_user", e.target.value)
                            }}
                        />
                    </label>
                </div>
            </div>  </div>
    )
}