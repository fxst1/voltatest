import { useState, useEffect, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";
import { error as logError } from "@tauri-apps/plugin-log";
import { VoltaTestAlert, utcNaiveToLocal } from "../types";
import { Panel } from "./base/panel";

type DetailFormProps = {
    alert: VoltaTestAlert;
    onClose: () => void;
    onDeleted: () => void;
};

function AlertDetailForm({ alert, onClose, onDeleted }: DetailFormProps) {
    const [loading, setLoading] = useState(false);

    async function handleDelete() {
        setLoading(true);
        try {
            await invoke("delete_alert", { id: alert.id });
            onDeleted();
        } catch (e: unknown) {
            logError("delete_alert error: " + e);
        } finally {
            setLoading(false);
        }
    }

    return (
        <div className="alarm-form">
            <div className="alarm-form-field">
                <label>Date &amp; Time</label>
                <input type="text" readOnly value={utcNaiveToLocal(alert.timestamp)} />
            </div>
            <div className="alarm-form-field">
                <label>Description</label>
                <input type="text" readOnly value={alert.description ?? ""} placeholder="—" />
            </div>
            <div className="alarm-form-field">
                <label>Data</label>
                <textarea readOnly value={alert.data} style={{ resize: "none", height: 56 }} />
            </div>
            <div className="alarm-form-actions">
                <button type="button" onClick={onClose}>Close</button>
                <button type="button" className="danger" onClick={handleDelete} disabled={loading}>
                    {loading ? "..." : "Delete"}
                </button>
            </div>
        </div>
    );
}

type PanelProps = {
    alerts: VoltaTestAlert[];
    onAlertDeleted: (alert: VoltaTestAlert) => void;
};

export function AlertPanel({ alerts, onAlertDeleted }: PanelProps) {
    const [locked, setLocked] = useState(true);
    const feedRef = useRef<HTMLDivElement>(null);

    useEffect(() => {
        if (locked && feedRef.current) {
            feedRef.current.scrollTop = feedRef.current.scrollHeight;
        }
    }, [alerts, locked]);

    const handleScroll = () => {
        const el = feedRef.current;
        if (!el) return;
        setLocked(el.scrollHeight - el.scrollTop - el.clientHeight < 40);
    };

    return (
        <Panel<VoltaTestAlert>
            title="Alerts"
            count={alerts.length}
            headerActions={
                <button
                    className={`alert-panel-follow${locked ? " alert-panel-follow--active" : ""}`}
                    onClick={() => setLocked(l => !l)}
                    title={locked ? "Pause scroll" : "Follow latest"}
                >
                    {locked ? "⬇️ Live" : "⏸️ Paused"}
                </button>
            }
            detailForm={(alert, onClose) => (
                <AlertDetailForm
                    alert={alert}
                    onClose={onClose}
                    onDeleted={() => { onAlertDeleted(alert); onClose(); }}
                />
            )}
            items={alerts}
            emptySentence="No alerts yet"
            onItem={alert => ({
                title: (
                    <>
                        <div className="alert-feed-time">{utcNaiveToLocal(alert.timestamp)}</div>
                        <div className="alert-feed-desc">{alert.description ?? <em>—</em>}</div>
                    </>
                ),
            })}
            onDeleteItem={onAlertDeleted}
            listRef={feedRef}
            onListScroll={handleScroll}
        />
    );
}
