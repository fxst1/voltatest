import { useState, ReactNode, RefObject, UIEventHandler } from "react";
import { ListItem, ListItemProps } from "./listitem";

export type PanelProps<T extends { id: string }> = {
    // Header
    title: string | ReactNode;
    count?: number;
    headerActions?: ReactNode;

    // Create form
    createLabel?: string;
    createForm?: (onClose: () => void) => ReactNode;

    // Detail / readonly form
    detailForm?: (item: T, onClose: () => void) => ReactNode;

    // List
    items: T[];
    loading?: boolean;
    emptySentence: string | ReactNode;
    onItem: (item: T) => ListItemProps;
    onDeleteItem?: (item: T) => void;

    // Scroll (live-follow)
    listRef?: RefObject<HTMLDivElement | null>;
    onListScroll?: UIEventHandler<HTMLDivElement>;
};

export function Panel<T extends { id: string }>({
    title,
    count,
    headerActions,
    createLabel,
    createForm,
    detailForm,
    items,
    loading,
    emptySentence,
    onItem,
    onDeleteItem,
    listRef,
    onListScroll,
}: PanelProps<T>) {
    const [showCreate, setShowCreate] = useState(false);
    const [openDetail, setOpenDetail] = useState<T | null>(null);

    function closeAll() {
        setShowCreate(false);
        setOpenDetail(null);
    }

    function handleOpenItem(item: T) {
        setShowCreate(false);
        setOpenDetail(item);
    }

    function handleToggleCreate() {
        setOpenDetail(null);
        setShowCreate(v => !v);
    }

    const formNode =
        showCreate && createForm ? createForm(closeAll) :
        openDetail && detailForm ? detailForm(openDetail, closeAll) :
        null;

    return (
        <div className="panel">
            <div className="panel-header">
                <span className="panel-title">
                    {title}
                    {count !== undefined && count > 0 && (
                        <span className="panel-count">{count}</span>
                    )}
                </span>
                <div className="panel-header-actions">
                    {headerActions}
                    {createLabel && createForm && (
                        <button
                            className={`panel-add-btn${showCreate ? " panel-add-btn--active" : ""}`}
                            onClick={handleToggleCreate}
                            title={createLabel}
                        >
                            {showCreate ? "✕" : "+"}
                        </button>
                    )}
                </div>
            </div>

            {formNode}

            <div className="panel-list" ref={listRef} onScroll={onListScroll}>
                {loading ? (
                    <div className="panel-empty">Loading...</div>
                ) : items.length === 0 ? (
                    <div className="panel-empty">{emptySentence}</div>
                ) : (
                    items.map(item => {
                        const props = onItem(item);
                        return (
                            <ListItem
                                key={item.id}
                                title={props.title}
                                onOpen={detailForm ? () => handleOpenItem(item) : undefined}
                                onDelete={onDeleteItem ? () => onDeleteItem(item) : undefined}
                            />
                        );
                    })
                )}
            </div>
        </div>
    );
}
