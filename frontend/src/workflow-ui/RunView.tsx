import { useEffect, useState } from 'react';
import { PlayCircle, CheckCircle2, XCircle, MinusCircle, History } from 'lucide-react';
import { runHistory, type RunRecord, type RunResult, type NodeRunStatus } from '../tauri-bridge';

type Props = {
    runResult: RunResult | null;
    isRunning: boolean;
    nodeLabels: Record<string, string>;
    workspacePath: string | null;
    pipelineId: string | null;
    runResultKey: number;
};

function StatusIcon({ status }: { status: NodeRunStatus['status'] }) {
    if (status === 'ok') return <CheckCircle2 size={13} className="run-row-ok" />;
    if (status === 'error') return <XCircle size={13} className="run-row-err" />;
    return <MinusCircle size={13} className="run-row-idle" />;
}

// Per-node result of the most recent run: status, row count, duration,
// and any error. Backed by the RunResult the engine returns (and the
// streamed stage_finished events App folds into it live during a run).
function relativeTime(iso: string): string {
    const then = Date.parse(iso);
    if (Number.isNaN(then)) return iso;
    const secs = Math.max(0, Math.round((Date.now() - then) / 1000));
    if (secs < 60) return `${secs}s ago`;
    const mins = Math.round(secs / 60);
    if (mins < 60) return `${mins}m ago`;
    const hours = Math.round(mins / 60);
    if (hours < 24) return `${hours}h ago`;
    return `${Math.round(hours / 24)}d ago`;
}

function PersistedRunSummary({ record }: { record: RunRecord }) {
    return (
        <div className="run-view">
            <div className={`run-summary run-summary-${record.status}`}>
                <span className="run-summary-status">{record.status}</span>
                <span className="run-summary-meta">
                    latest persisted run · {record.node_count} node
                    {record.node_count === 1 ? '' : 's'} · {record.duration_ms} ms
                </span>
            </div>
            {record.error ? (
                <pre className="run-error-body">
                    {record.category ? (
                        <span className="run-error-category">{record.category}</span>
                    ) : null}
                    {record.error}
                </pre>
            ) : null}
            <table className="run-table">
                <thead>
                    <tr>
                        <th></th>
                        <th>When</th>
                        <th>Trigger</th>
                        <th className="run-num">Rows</th>
                        <th className="run-num">Time</th>
                    </tr>
                </thead>
                <tbody>
                    <tr className={`run-row run-row-${record.status}`}>
                        <td>
                            <StatusIcon status={record.status as NodeRunStatus['status']} />
                        </td>
                        <td>
                            <div className="runs-when" title={record.at}>
                                {relativeTime(record.at)}
                            </div>
                            <div className="run-node-error">
                                Summary loaded from workspace history. Run again to populate
                                per-node output for this browser session.
                            </div>
                        </td>
                        <td>{record.trigger}</td>
                        <td className="run-num">{record.rows.toLocaleString()}</td>
                        <td className="run-num">{record.duration_ms} ms</td>
                    </tr>
                </tbody>
            </table>
        </div>
    );
}

// Per-node result of the most recent run: status, row count, duration,
// and any error. Backed by the RunResult the engine returns (and the
// streamed stage_finished events App folds into it live during a run).
// If no in-memory result exists after reload/navigation, fall back to the
// persisted run-history summary so the tab does not look falsely empty.
export default function RunView({
    runResult,
    isRunning,
    nodeLabels,
    workspacePath,
    pipelineId,
    runResultKey,
}: Props) {
    const [latestRecord, setLatestRecord] = useState<RunRecord | null>(null);
    const [historyLoaded, setHistoryLoaded] = useState(false);

    useEffect(() => {
        let alive = true;
        setLatestRecord(null);
        setHistoryLoaded(false);
        if (runResult || !pipelineId) {
            setHistoryLoaded(true);
            return;
        }
        runHistory(workspacePath, pipelineId).then(records => {
            if (alive) {
                setLatestRecord(records[0] ?? null);
                setHistoryLoaded(true);
            }
        });
        return () => {
            alive = false;
        };
    }, [workspacePath, pipelineId, runResult, runResultKey]);

    if (!runResult) {
        if (latestRecord) return <PersistedRunSummary record={latestRecord} />;
        return (
            <div className="empty-state">
                {historyLoaded ? (
                    <PlayCircle size={32} strokeWidth={1.4} className="empty-icon" />
                ) : (
                    <History size={32} strokeWidth={1.4} className="empty-icon" />
                )}
                <div className="empty-title">Run output</div>
                <div className="empty-desc">
                    {historyLoaded
                        ? 'Per-node row counts, timings, and errors from the last run will appear here. Press Run to execute the pipeline.'
                        : 'Loading the latest persisted run from workspace history.'}
                </div>
            </div>
        );
    }

    const entries = Object.entries(runResult.nodes);

    return (
        <div className="run-view">
            <div className={`run-summary run-summary-${runResult.status}`}>
                <span className="run-summary-status">
                    {isRunning ? 'running' : runResult.status}
                </span>
                <span className="run-summary-meta">
                    {entries.length} node{entries.length === 1 ? '' : 's'} ·{' '}
                    {runResult.duration_ms} ms
                </span>
            </div>
            {runResult.error ? (
                <pre className="run-error-body">
                    {runResult.category ? (
                        <span className="run-error-category">{runResult.category}</span>
                    ) : null}
                    {runResult.error}
                </pre>
            ) : null}
            {runResult.messages && runResult.messages.length > 0 ? (
                <ul className="run-messages">
                    {runResult.messages.map((m, i) => (
                        <li key={i} className={`run-message run-message-${m.level}`}>
                            <span className="run-message-level">{m.level}</span>
                            <span className="run-message-node">
                                {nodeLabels[m.node_id] ?? m.node_id}
                            </span>
                            <span className="run-message-text">{m.message}</span>
                        </li>
                    ))}
                </ul>
            ) : null}
            <table className="run-table">
                <thead>
                    <tr>
                        <th></th>
                        <th>Node</th>
                        <th>Kind</th>
                        <th className="run-num">Rows</th>
                        <th className="run-num">Time</th>
                    </tr>
                </thead>
                <tbody>
                    {entries.map(([nodeId, st]) => (
                        <tr key={nodeId} className={`run-row run-row-${st.status}`}>
                            <td>
                                <StatusIcon status={st.status} />
                            </td>
                            <td>
                                <div className="run-node-label">
                                    {nodeLabels[nodeId] ?? nodeId}
                                </div>
                                {st.error ? (
                                    <div className="run-node-error">
                                        {st.category ? (
                                            <span className="run-error-category">
                                                {st.category}
                                            </span>
                                        ) : null}
                                        {st.error}
                                    </div>
                                ) : null}
                            </td>
                            <td>{st.kind ?? ''}</td>
                            <td className="run-num">
                                {st.rows != null ? st.rows.toLocaleString() : '-'}
                            </td>
                            <td className="run-num">
                                {st.duration_ms != null ? `${st.duration_ms} ms` : '-'}
                            </td>
                        </tr>
                    ))}
                </tbody>
            </table>
        </div>
    );
}
