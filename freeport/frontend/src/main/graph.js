import { getGraph, getMockGraphSimple } from "../api_access/get_graph";
import { normalizeData } from "../data/normalize";
import { Input, SubmitButton } from "../gui/input";
import { renderGraph } from "../gui/render_graph";
import { clearChildNodes } from "../gui/util";

export const graphMain = async (GlobalConfig) => {

    const outputElem = document.getElementById(GlobalConfig.outputElemId);

    const run = () => {
        clearChildNodes(GlobalConfig.outputElemId);

        outputElem.innerText = "Loading...";
        
        getGraph(
            Input.graphTopN.parseValue(),
            Input.limit.parseValue(),
            Input.depth.parseValue(),
            Input.weightDecayMode.parseValue(),
        )
        // getMockGraphSimple()
        .then((dataGraph) => {
            outputElem.innerText = "";
            
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
        })
        .catch(console.error);
    };
    
    SubmitButton.addEventListener("click", run);
};