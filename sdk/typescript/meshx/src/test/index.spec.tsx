/**
 * @vitest-environment jsdom
 */

declare module "react" {
    // eslint-disable-next-line no-var
    var unstable_act: (cb: () => void) => Promise<void> | void
}

import { useEffect, useState, unstable_act as act } from "react"
import { WorkerReconciler, HostContext } from "../lib/hostConfig"

const host: HostContext = {
    tag: "root",
    onCommit: (c) => {
        console.log("W: commit", c)
    },
    generateId: (() => {
        let id = 0
        return () => id++
    })(),
}

const portal: HostContext = {
    tag: "portal",
    onCommit: (c) => {
        console.log("W: commit", c)
    },
    generateId: (() => {
        let id = 0
        return () => id++
    })(),
}

declare global {
    // eslint-disable-next-line @typescript-eslint/no-namespace
    namespace JSX {
        interface ScenariosAttributes {
            foo: string
        }

        // TypeScript used this interface to know what kind of intrinsic elements can be rendered.
        // https://www.typescriptlang.org/docs/handbook/jsx.html#type-checking
        interface IntrinsicElements {
            scenarios: ScenariosAttributes
            mxtext: { type: "header"; children: any }
            mxbutton: { onClick: () => void; children: any }
        }
    }
}

function App() {
    const [count, setCount] = useState(0)

    useEffect(() => {
        const timeout = setTimeout(() => {
            setCount(1)
        }, 1000)

        return () => clearTimeout(timeout)
    }, [])

    return (
        <>
            {count}
            {WorkerReconciler.createPortal(<mxtext type="header">test</mxtext>, portal)}
            <mxtext type="header">Nothing to see here</mxtext>
            <mxbutton
                onClick={() => {
                    return
                }}>
                zes
            </mxbutton>
        </>
    )
}

describe("sdkTypescriptMeshx", async () => {
    it("should work", async () => {
        vitest.useFakeTimers({ shouldAdvanceTime: true })

        await act(() => {
            WorkerReconciler.render(<App />, host)
        })

        await act(() => {
            vitest.runAllTimers()
        })
    })
})
