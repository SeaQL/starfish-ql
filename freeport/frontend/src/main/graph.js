import { getGraphSimple, getMockGraphSimple } from "../api_access/get_graph";
import { normalizeData } from "../data/normalize";
import { Input } from "../gui/input";
import { renderGraph } from "../gui/render_graph";
import { clearChildNodes } from "../gui/util";

export const graphMain = async(GlobalConfig) => {

    const run = () => {
        clearChildNodes(GlobalConfig.outputElemId);
        
        // getGraphSimple(
        //     Input.graphTopN.parseInt(),
        //     Input.limit.parseInt(),
        //     Input.depth.parseInt(),
        // )
        getMockGraphSimple()
        .then((dataGraph) => {
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
    run();
    
    Input.elems.forEach((elem) => {
        if (elem === null) {
            return;
        }
        elem.addEventListener("focusout", run);
    });
};