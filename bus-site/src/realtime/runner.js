import {ViteNodeRunner} from "vite-node/client";
import {workerData} from "node:worker_threads";
import {installSourcemapsSupport} from "vite-node/source-map";
import {ViteNodeServer} from "vite-node/server";
import {createServer} from "vite";
import { svelte } from "@sveltejs/vite-plugin-svelte";

async function run() {
    /** @type {{path: string, data: any}} */
    let data = workerData

    // create vite server
    const server = await createServer({
        mode: "production",
        optimizeDeps: {
            disabled: true,
        },
        plugins: svelte({
            prebundleSvelteLibraries: false
        })
    })
    await server.pluginContainer.buildStart({})

    const node = new ViteNodeServer(server)

    installSourcemapsSupport({
        getSourceMap: source => node.getSourceMap(source)
    })

    const runner = new ViteNodeRunner({
        root: server.config.root,
        base: server.config.base,
        async fetchModule(id) {
            return node.fetchModule(id)
        },
        async resolveId(id, importer) {
            return node.resolveId(id, importer)
        },
    })

    await runner.executeFile(data.path)
}

run()