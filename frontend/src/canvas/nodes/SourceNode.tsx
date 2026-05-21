import { Handle, Position, type Node, type NodeProps } from '@xyflow/react';
import type { DuckleNodeData } from '../../pipeline-types';

type SourceNodeType = Node<DuckleNodeData, 'source'>;

export default function SourceNode({ data, selected }: NodeProps<SourceNodeType>) {
    return (
        <div className={'node node-source' + (selected ? ' is-selected' : '')}>
            <div className="node-kind">source</div>
            <div className="node-label">{data.label}</div>
            {data.subtitle ? <div className="node-subtitle">{data.subtitle}</div> : null}
            <Handle type="source" position={Position.Right} />
        </div>
    );
}
