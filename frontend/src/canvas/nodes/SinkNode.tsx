import { Handle, Position, type Node, type NodeProps } from '@xyflow/react';
import type { DuckleNodeData } from '../../pipeline-types';

type SinkNodeType = Node<DuckleNodeData, 'sink'>;

export default function SinkNode({ data, selected }: NodeProps<SinkNodeType>) {
    return (
        <div className={'node node-sink' + (selected ? ' is-selected' : '')}>
            <div className="node-kind">sink</div>
            <div className="node-label">{data.label}</div>
            {data.subtitle ? <div className="node-subtitle">{data.subtitle}</div> : null}
            <Handle type="target" position={Position.Left} />
        </div>
    );
}
