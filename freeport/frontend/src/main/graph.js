import { getGraph, getMockGraphSimple } from "../api_access/get_graph";
import { normalizeData } from "../data/normalize";
import { Input, SubmitButton } from "../gui/input";
import { renderGraph } from "../gui/render_graph";
import { clearChildNodes } from "../gui/util";

export const graphMain = async (GlobalConfig, callback) => {

    clearChildNodes(GlobalConfig.outputElemId);
    
    getGraph(
        Input.graphTopN.parseValue(),
        Input.limit.parseValue(),
        Input.depth.parseValue(),
        Input.weightDecayMode.parseValue(),
    )
    .then((dataGraph) => runRenderGraph(GlobalConfig, dataGraph))
    .then(() => callback())
    .catch(console.error);
};

export const graphMock = async (GlobalConfig) => {

    clearChildNodes(GlobalConfig.outputElemId);
    
    getMockGraphSimple()
    .then((dataGraph) => runRenderGraph(GlobalConfig, dataGraph))
    .catch(console.error);
};

const runRenderGraph = async (GlobalConfig, dataGraph) => {
    normalizeData(
        dataGraph,
        (data) => data.nodes.map((node) => node.weight),
        (data, normalizedWeights) => {
            normalizedWeights.forEach((normalizedWeight, i) => {
                data.nodes[i].weight = normalizedWeight;
            });
        },
        {
            newMin: GlobalConfig.minWeight,
            newMax: GlobalConfig.maxWeight,
        }
    );

    renderGraph(
        dataGraph,
        document.getElementById(GlobalConfig.outputElemId),
        {
            textDelimiters: "-+",
            minFontSize: GlobalConfig.minWeight,
        }
    );
};