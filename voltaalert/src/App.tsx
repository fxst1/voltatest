import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { debug, error, info } from "@tauri-apps/plugin-log";
import { convertDataTuple, ListAlertCommandArgs, VoltaTestAlert } from "./types";
import { AlertPanel } from "./components/alert-feed";
import { AlarmPanel } from "./components/alarm-panel";
import "./App.css";

type Tab = "alarms" | "alerts";

function App() {
    const [alerts, setAlerts] = useState<VoltaTestAlert[]>([]);
    const [activeTab, setActiveTab] = useState<Tab>("alerts");

    useEffect(() => {
        const unlisten = load().then(() => {
            info("Start listening...");
            return listen<[string, Partial<VoltaTestAlert>]>("alert", (e) => {
                debug(`GOT alert: ${JSON.stringify(e.payload)}`);
                setAlerts(prev => [...prev, convertDataTuple<VoltaTestAlert>(e.payload)]);
            });
        });
        return () => { unlisten.then(fn => fn()); };
    }, []);

    async function load() {
        try {
            info("Loading previous alerts");
            let args: ListAlertCommandArgs = { lastFetchedId: null };
            while (true) {
                const result: Array<[string, any]> = await invoke("list_alerts", args);
                if (result.length === 0) break;
                args.lastFetchedId = result[result.length - 1][0];
                setAlerts(prev => [...prev, ...result.map(convertDataTuple<VoltaTestAlert>)]);
            }
        } catch (e) {
            error("list_alerts error: " + e);
        }
    }

    return (
        <div className="app-layout">

            <main className="app-main">
                <div className="tab-bar">
                    <button
                        className={`tab-btn${activeTab === "alarms" ? " tab-btn--active" : ""}`}
                        onClick={() => setActiveTab("alarms")}
                    >
                        Alarms
                    </button>
                    <button
                        className={`tab-btn${activeTab === "alerts" ? " tab-btn--active" : ""}`}
                        onClick={() => setActiveTab("alerts")}
                    >
                        Alertes
                    </button>
                </div>

                {activeTab === "alarms" ? (
                    <AlarmPanel />
                ) : (
                    <AlertPanel
                        alerts={alerts}
                        onAlertDeleted={alert =>
                            setAlerts(prev => prev.filter(a => a !== alert))
                        }
                    />
                )}
            </main>

        </div>
    );
}

export default App;
