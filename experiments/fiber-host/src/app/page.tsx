import dynamic from "next/dynamic"

const Host = dynamic(() => import("../components/Host"), { ssr: false })

export default function Index() {
    /*
     * Replace the elements below with your own.
     *
     * Note: The corresponding styles are in the ./index.tailwind file.
     */
    return (
        <div>
            <Host></Host>
        </div>
    )
}
