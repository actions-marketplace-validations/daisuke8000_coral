import { useCallback } from 'react';
import dagre from 'dagre';
import type { Node, Edge } from '@xyflow/react';

const BASE_WIDTH = 200;

/** Estimate node dimensions from data content. */
function getNodeDimensions(node: Node): { width: number; height: number } {
  const details = node.data?.details as
    | { kind: string; methods?: unknown[]; fields?: unknown[]; values?: unknown[] }
    | undefined;

  if (!details) {
    return { width: BASE_WIDTH, height: 100 };
  }

  switch (details.kind) {
    case 'Service':
      return { width: BASE_WIDTH, height: 80 + (details.methods?.length ?? 0) * 28 };
    case 'Message':
      return { width: BASE_WIDTH, height: 80 + (details.fields?.length ?? 0) * 24 };
    case 'Enum':
      return { width: BASE_WIDTH, height: 80 + (details.values?.length ?? 0) * 24 };
    default:
      return { width: BASE_WIDTH, height: 100 };
  }
}

export function useAutoLayout() {
  const getLayoutedNodes = useCallback((
    nodes: Node[],
    edges: Edge[],
    direction: 'TB' | 'LR' = 'TB'
  ): Node[] => {
    const g = new dagre.graphlib.Graph();
    g.setGraph({ rankdir: direction, nodesep: 80, ranksep: 100 });
    g.setDefaultEdgeLabel(() => ({}));

    const dimensionsMap = new Map<string, { width: number; height: number }>();

    nodes.forEach((node) => {
      const dims = getNodeDimensions(node);
      dimensionsMap.set(node.id, dims);
      g.setNode(node.id, { width: dims.width, height: dims.height });
    });

    edges.forEach((edge) => {
      g.setEdge(edge.source, edge.target);
    });

    dagre.layout(g);

    return nodes.map((node) => {
      const pos = g.node(node.id);
      const dims = dimensionsMap.get(node.id) ?? { width: BASE_WIDTH, height: 100 };
      return {
        ...node,
        position: {
          x: pos.x - dims.width / 2,
          y: pos.y - dims.height / 2,
        },
      };
    });
  }, []);

  return { getLayoutedNodes };
}
