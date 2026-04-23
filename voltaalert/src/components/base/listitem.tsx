import { useState, ReactNode } from "react";

export type ListItemProps = {
    title: string | ReactNode;
    onDelete?: () => void;
    onOpen?: () => void;
};

export function ListItem({ title, onDelete, onOpen }: ListItemProps) {
    const [confirming, setConfirming] = useState(false);

    return (
        <div className="list-item" onClick={onOpen} style={onOpen ? { cursor: "pointer" } : undefined}>
            <div className="list-item-main">
                {title}
            </div>
            {onDelete && (
                <div className="list-item-actions">
                    {confirming ? (
                        <>
                            <button
                                className="danger list-confirm-btn"
                                onClick={e => { e.stopPropagation(); onDelete(); }}
                            >
                                Delete
                            </button>
                            <button
                                className="list-cancel-btn"
                                onClick={e => { e.stopPropagation(); setConfirming(false); }}
                            >
                                No
                            </button>
                        </>
                    ) : (
                        <button
                            className="list-delete-btn"
                            onClick={e => { e.stopPropagation(); setConfirming(true); }}
                            title="Delete"
                        >
                            ✕
                        </button>
                    )}
                </div>
            )}
        </div>
    );
}

export type ListHandlersProps<T extends {id: string}> = {
    emptySentence: string | ReactNode;
    onItem: (item: T) => ListItemProps;
    onDeleteItem?: (item: T) => void
    onOpenItem?: (item: T) => void
}
export type ListProps<T extends {id: string}> = {
    items: T[];
} & ListHandlersProps<T>;

export function BaseList<T extends {id: string}>({ items, emptySentence, onItem, onDeleteItem, onOpenItem }: ListProps<T>) {
    return (
        <div className="list">
            {items.length === 0 ? (
                <div className="list-empty">{emptySentence}</div>
            ) : (
                items.map(item => {
                    const props = onItem(item);
                    return (
                        <ListItem
                            key={item.id}
                            title={props.title}
                            onOpen={onOpenItem ? () => onOpenItem(item) : undefined}
                            onDelete={onDeleteItem ? () => onDeleteItem(item) : undefined }
                         />
                    )
                })
            )}
        </div>
    );
}
