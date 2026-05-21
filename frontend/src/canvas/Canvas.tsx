import {
    ReactFlow,
    ReactFlowProvider,
    Background,
    Controls,
    MiniMap,
    type Connection,
    type Edge,
    type EdgeChange,
    type Node,
    type NodeChange,
    type OnSelectionChangeParams,
} from '@xyflow/react';
import '@xyflow/react/dist/style.css';
import SourceNode from './nodes/SourceNode';
import TransformNode from './nodes/TransformNode';
import SinkNode from './nodes/SinkNode';
import type { DuckleNodeData } from '../pipeline-types';

const nodeTypes = {
    source: SourceNode,
    transform: TransformNode,
    sink: SinkNode,
};

type Props = {
    nodes: Node<DuckleNodeData>[];
    edges: Edge[];
    onNodesChange: (changes: NodeChange[]) => void;
    onEdgesChange: (changes: EdgeChange[]) => void;
    onConnect: (connection: Connection) => void;
    onSelectionChange: (params: OnSelectionChangeParams) => void;
};

function CanvasInner({
    nodes,
    edges,
    onNodesChange,
    onEdgesChange,
    onConnect,
    onSelectionChange,
}: Props) {
    return (
        <ReactFlow
            nodes={nodes}
            edges={edges}
            onNodesChange={onNodesChange}
            onEdgesChange={onEdgesChange}
            onConnect={onConnect}
            onSelectionChange={onSelectionChange}
            nodeTypes={nodeTypes}
            fitView
            colorMode="dark"
        >
            <Background gap={16} />
            <MiniMap pannable zoomable />
            <Controls />
        </ReactFlow>
    );
}

export default function Canvas(props: Props) {
    return (
        <ReactFlowProvider>
            <CanvasInner {...props} />
        </ReactFlowProvider>
    );
}
