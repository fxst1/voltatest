export type VoltaTestAlert = {
    id: string;
    description: string | null;
    timestamp: string;
    data: string;
};

export type VoltaTestAlarm = {
    id: string;
    description: string;
    kind_id: string;
};

export type AlarmKindId = "alarm-clock" | "pattern" | "always";
export type PatternOperatorKind = "Eq" | "Neq" | "Between" | "Contains";

export type UpdateAlertCommandArgs = {
    id: string
    description: string | null;
    enabled: boolean;
};

export type ListAlertCommandArgs = {
    lastFetchedId: string | null;
};

// Convert tuple EntityData(string, T) into a concrete object
export function convertDataTuple<T>(value: [string, Partial<T>]): T {
    return {
        id: value[0],
        ...value[1]
    } as T
}

// Convert a NaiveDateTime UTC string from the backend (no "Z" suffix)
export function utcNaiveToLocal(utcStr: string): string {
    const d = new Date(utcStr + "Z");
    const isToday = new Date().toDateString() === d.toDateString();
    return isToday ? d.toLocaleTimeString() : d.toLocaleString();
}
