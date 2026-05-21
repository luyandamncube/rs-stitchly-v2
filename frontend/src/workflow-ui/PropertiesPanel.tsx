import { useState } from 'react';
import type { Node } from '@xyflow/react';
import type { Column, DuckleNodeData } from '../pipeline-types';
import SchemaEditor from './SchemaEditor';

type TabId = 'basic' | 'schema' | 'advanced' | 'validation';

type Props = {
    selected: Node<DuckleNodeData> | null;
    onUpdate: (id: string, patch: Partial<DuckleNodeData>) => void;
};

const TABS: { id: TabId; label: string }[] = [
    { id: 'basic', label: 'Basic' },
    { id: 'schema', label: 'Schema' },
    { id: 'advanced', label: 'Advanced' },
    { id: 'validation', label: 'Validation' },
];

const KIND_LABEL: Record<string, string> = {
    source: 'Source',
    transform: 'Transform',
    sink: 'Sink',
};

const KIND_COLOR: Record<string, string> = {
    source: '#7ee787',
    transform: '#58a6ff',
    sink: '#ffa657',
};

export default function PropertiesPanel({ selected, onUpdate }: Props) {
    const [tab, setTab] = useState<TabId>('basic');

    if (!selected) {
        return (
            <aside className="properties">
                <div className="properties-empty">
                    <svg
                        width="40"
                        height="40"
                        viewBox="0 0 24 24"
                        fill="none"
                        stroke="currentColor"
                        strokeWidth="1.6"
                        strokeLinecap="round"
                        strokeLinejoin="round"
                        aria-hidden="true"
                    >
                        <circle cx="12" cy="12" r="9" />
                        <line x1="12" y1="8" x2="12" y2="12" />
                        <circle cx="12" cy="16" r="0.5" fill="currentColor" />
                    </svg>
                    <div className="properties-empty-title">Nothing selected</div>
                    <div className="properties-empty-desc">
                        Click a node on the canvas to edit its configuration, schema, and validation
                        rules.
                    </div>
                </div>
            </aside>
        );
    }

    const kind = (selected.type ?? 'transform') as string;
    const data = selected.data;
    const schema = data.schema ?? [];

    const setLabel = (label: string) => onUpdate(selected.id, { label });
    const setSubtitle = (subtitle: string) => onUpdate(selected.id, { subtitle });
    const setSchema = (columns: Column[]) => onUpdate(selected.id, { schema: columns });

    return (
        <aside className="properties">
            <div className="properties-header">
                <div className="properties-kind-row">
                    <span
                        className="properties-kind-dot"
                        style={{ background: KIND_COLOR[kind] ?? '#666' }}
                        aria-hidden="true"
                    />
                    <span className="properties-kind">{KIND_LABEL[kind] ?? kind}</span>
                    <span className="properties-id">#{selected.id}</span>
                </div>
                <input
                    type="text"
                    className="properties-name-input"
                    value={data.label}
                    onChange={e => setLabel(e.target.value)}
                    placeholder="Component name"
                    spellCheck={false}
                />
            </div>

            <div className="properties-tabs" role="tablist">
                {TABS.map(t => (
                    <button
                        key={t.id}
                        type="button"
                        role="tab"
                        aria-selected={tab === t.id}
                        className="properties-tab"
                        onClick={() => setTab(t.id)}
                    >
                        {t.label}
                    </button>
                ))}
            </div>

            <div className="properties-content">
                {tab === 'basic' ? (
                    <div className="properties-section">
                        <Field label="Subtitle">
                            <input
                                type="text"
                                className="field-input"
                                value={data.subtitle ?? ''}
                                onChange={e => setSubtitle(e.target.value)}
                                placeholder="Short description"
                            />
                        </Field>
                        {data.componentId ? (
                            <Field label="Component">
                                <div className="field-readonly">{data.componentId}</div>
                            </Field>
                        ) : null}
                        <div className="properties-hint">
                            Component-specific properties (paths, predicates, credentials) will
                            render here once the Rust plugin SDK is wired up.
                        </div>
                    </div>
                ) : null}

                {tab === 'schema' ? (
                    <div className="properties-section">
                        <SchemaEditor columns={schema} onChange={setSchema} />
                    </div>
                ) : null}

                {tab === 'advanced' ? (
                    <div className="properties-section">
                        <div className="properties-placeholder">
                            <div className="properties-placeholder-title">Advanced settings</div>
                            <div className="properties-placeholder-desc">
                                Buffering, parallelism, retry policy, custom partitioning, encoding
                                options, and other rarely-touched knobs will live here.
                            </div>
                        </div>
                    </div>
                ) : null}

                {tab === 'validation' ? (
                    <div className="properties-section">
                        <div className="validation-summary validation-ok">
                            <span className="validation-icon" aria-hidden="true">
                                ✓
                            </span>
                            <span>No issues detected for this node.</span>
                        </div>
                        <div className="properties-hint">
                            Schema mismatches, missing required properties, and engine
                            compatibility warnings will surface here.
                        </div>
                    </div>
                ) : null}
            </div>
        </aside>
    );
}

function Field({ label, children }: { label: string; children: React.ReactNode }) {
    return (
        <label className="field">
            <span className="field-label">{label}</span>
            {children}
        </label>
    );
}
