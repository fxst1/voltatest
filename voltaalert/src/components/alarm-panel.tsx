import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { error as logError } from "@tauri-apps/plugin-log";
import { AlarmKindId, PatternOperatorKind, VoltaTestAlarm } from "../types";
import { Panel } from "./base/panel";

type CreateFormProps = {
    onCreated: (raw: [string, any]) => void;
    onCancel: () => void;
};

function AlarmCreateForm({ onCreated, onCancel }: CreateFormProps) {
    const [description, setDescription] = useState("");
    const [kind, setKind] = useState<AlarmKindId>("alarm-clock");
    const [triggerAt, setTriggerAt] = useState("");
    const [operator, setOperator] = useState<PatternOperatorKind>("Eq");
    const [opValue, setOpValue] = useState("");
    const [loading, setLoading] = useState(false);
    const [err, setErr] = useState<string | null>(null);

    function buildConfig(): unknown {
        if (kind === "alarm-clock") {
            const utcStr = new Date(triggerAt).toISOString().slice(0, 19);
            return { trigger_at: utcStr, passed: false };
        }
        if (kind === "always") return {};
        const encode = (s: string) => Array.from(new TextEncoder().encode(s));
        if (operator === "Eq")      return { operator: { Eq: encode(opValue) } };
        if (operator === "Neq")     return { operator: { Neq: encode(opValue) } };
        throw new Error("Unexpected Kind or Operator");
    }

    async function handleSubmit(e: React.SubmitEvent) {

        e.preventDefault();

        if (!description.trim()) { setErr("Description is required"); return; }
        if (kind === "alarm-clock" && !triggerAt) { setErr("Date of trigger is required"); return; }

        setLoading(true);
        setErr(null);

        try {
            const result = await invoke<[string, any]>("create_alarm", {
                description: description.trim(),
                kind,
                config: buildConfig(),
            });
            onCreated(result);
        } catch (e: unknown) {
            logError("create_alarm error: " + e);
            setErr(String(e));
        } finally {
            setLoading(false);
        }
    }

    return (
        <form className="alarm-form" onSubmit={e => handleSubmit(e)}>
            <div className="alarm-form-field">
                <label>Description</label>
                <input
                    value={description}
                    onChange={e => setDescription(e.target.value)}
                    placeholder="Alarm name (used for alert description)"
                    autoFocus
                />
            </div>

            <div className="alarm-form-field">
                <label>Type</label>
                <select value={kind} onChange={e => setKind(e.target.value as AlarmKindId)}>
                    <option value="alarm-clock">Clock</option>
                    <option value="pattern">Pattern</option>
                    <option value="always">Always</option>
                </select>
            </div>

            {kind === "alarm-clock" && (
                <div className="alarm-form-field">
                    <label>Trigger at</label>
                    <input
                        type="datetime-local"
                        value={triggerAt}
                        onChange={e => setTriggerAt(e.target.value)}
                    />
                </div>
            )}

            {kind === "pattern" && (
                <>
                    <div className="alarm-form-field">
                        <label>Operator</label>
                        <select value={operator} onChange={e => setOperator(e.target.value as PatternOperatorKind)}>
                            <option value="Eq">Received data must be equals to</option>
                            <option value="Neq">Received data must differs than</option>
                        </select>
                    </div>

                    {(operator === "Eq" || operator === "Neq") && (
                        <div className="alarm-form-field">
                            <label>Value to compare</label>
                            <input
                                value={opValue}
                                onChange={e => setOpValue(e.target.value)}
                                placeholder="Text to compare..."
                            />
                        </div>
                    )}
                </>
            )}

            {err && <div className="alarm-form-error">{err}</div>}

            <div className="alarm-form-actions">
                <button type="button" onClick={onCancel} disabled={loading}>Cancel</button>
                <button type="submit" className="primary" disabled={loading}>
                    {loading ? "Creating..." : "Create"}
                </button>
            </div>
        </form>
    );
}

export function AlarmPanel() {
    const [alarms, setAlarms] = useState<VoltaTestAlarm[]>([]);
    const [loading, setLoading] = useState(true);

    useEffect(() => { loadAlarms(); }, []);

    async function loadAlarms() {
        setLoading(true);
        try {
            let lastFetchedId: string | null = null;
            const all: VoltaTestAlarm[] = [];
            while (true) {
                const page: Array<[string, any]> = await invoke("list_alarms", { lastFetchedId });
                if (page.length === 0) break;
                lastFetchedId = page[page.length - 1][0];
                all.push(...page.map(([id, d]) => ({ id, description: d.description, kind_id: d.kind_id })));
            }
            setAlarms(all);
        } catch (e: unknown) {
            logError("list_alarms error: " + e);
        } finally {
            setLoading(false);
        }
    }

    async function handleDelete(alarm: VoltaTestAlarm) {
        try {
            await invoke("delete_alarm", { id: alarm.id });
            setAlarms(prev => prev.filter(a => a.id !== alarm.id));
        } catch (e: unknown) {
            logError("delete_alarm error: " + e);
        }
    }

    return (
        <Panel<VoltaTestAlarm>
            title="Alarms"
            count={alarms.length}
            createLabel="New alarm"
            createForm={onClose => (
                <AlarmCreateForm
                    onCreated={([id, d]) => {
                        setAlarms(prev => [...prev, { id, description: d.description, kind_id: d.kind_id }]);
                        onClose();
                    }}
                    onCancel={onClose}
                />
            )}
            items={alarms}
            loading={loading}
            emptySentence="No alarm yet"
            onItem={alarm => ({
                title: (
                    <div className="alarm-item-content">
                        <div className={`alarm-kind-badge alarm-kind-badge--${alarm.kind_id}`}>
                            {alarm.kind_id}
                        </div>
                        <div className="alarm-item-desc">{alarm.description}</div>
                    </div>
                ),
            })}
            onDeleteItem={handleDelete}
        />
    );
}
