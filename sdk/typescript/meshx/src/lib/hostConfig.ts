/* eslint-disable @typescript-eslint/no-empty-function */
import ReactReconciler, { HostConfig, OpaqueHandle } from "react-reconciler"
function assert(condition: any, msg?: string): asserts condition {
    if (!condition) {
        throw new Error(msg)
    }
}

export enum NodeType {
    view = "mxview",
    text = "mxtext",
    btn = "mxbutton",
}

type TextProps = {
    color?: string
    type?: "header" | "paragraph" | "span"
    children?: string | number
}

type ViewProps = {
    border: number
}

type ButtonProps = {
    onClick: () => void
}

type ViewNodeInstance = {
    tag: NodeType.view
    border?: number
    children: AnyInstance[]
}

export type BtnNodeInstance = {
    tag: NodeType.btn
    id: number
    onClick: () => void
    children: AnyInstance[]
}

type TextNodeInstance = {
    tag: NodeType.text
    color?: string
    type?: "header" | "paragraph" | "span"
    text: string | number | undefined
    children: AnyInstance[]
}

export type TextInstance = TextNodeInstance

export type Instance = ViewNodeInstance | TextNodeInstance | BtnNodeInstance

// let's make them the same for now
type PublicInstance = Instance | TextInstance

export type AnyInstance = Instance | TextInstance

export type Container = {
    hostContext: HostContext
    children: (Instance | TextInstance)[]
}

export type HostContext = {
    tag: string
    onCommit: (container: Container) => void
    generateId: () => number
}

type Props = TextProps | ViewProps | ButtonProps

type UpdatePayload = {
    old: Props
    new: Props
}

type NoTimeout = -1
type TimeoutHandle = number

type SuspenseInstance = never
type HydratableInstance = never
type ChildSet = never

type MyConfig = HostConfig<
    NodeType,
    Props,
    Container,
    Instance,
    TextInstance,
    SuspenseInstance,
    HydratableInstance,
    PublicInstance,
    HostContext,
    UpdatePayload,
    ChildSet,
    TimeoutHandle,
    NoTimeout
>

const getPublicInstance: MyConfig["getPublicInstance"] = (instance: Instance | TextInstance): PublicInstance => {
    return instance
}

const getRootHostContext: MyConfig["getRootHostContext"] = (container: Container): HostContext => container.hostContext

const getChildHostContext: MyConfig["getChildHostContext"] = (
    parentHostContext: HostContext,
    type: NodeType,
    rootContainerInstance: Container
): HostContext => parentHostContext

const prepareForCommit: MyConfig["prepareForCommit"] = (container) => {
    // noop
    return null
}

const resetAfterCommit: MyConfig["resetAfterCommit"] = (container) => {
    // console.log("resetAfterCommit", JSON.stringify(container, null, 2));
    container.hostContext.onCommit(container)
}

const shouldSetTextContent: MyConfig["shouldSetTextContent"] = (type: NodeType, props: Props): boolean =>
    // false;
    type === NodeType.text && typeof (props as TextProps).children === "string"

const createInstance: MyConfig["createInstance"] = (
    type: NodeType,
    props: Props,
    rootContainerInstance: Container,
    hostContext: HostContext,
    internalInstanceHandle: OpaqueHandle
): Instance => {
    switch (type) {
        case NodeType.view: {
            // TODO check props
            const { border } = props as ViewProps
            const view: ViewNodeInstance = { tag: type, border, children: [] }
            return view
        }
        case NodeType.btn: {
            // TODO check props
            const { onClick } = props as ButtonProps
            const btn: BtnNodeInstance = {
                tag: type,
                onClick,
                id: hostContext.generateId(),
                children: [],
            }
            return btn
        }
        case NodeType.text: {
            // TODO check props
            const { children, color, type: textType } = props as TextProps
            //   console.log("createInstance", children);
            //   assert(
            //     typeof children === "string" ||
            //       typeof children === "number" ||
            //       typeof children === "undefined",
            //     "why did we get non literals in here?"
            //   );
            const text: TextNodeInstance = {
                tag: type,
                type: textType,
                color,
                text: typeof children === "string" || typeof children === "number" ? children : undefined,
                children: [],
            }
            return text
        }
    }
}

const createTextInstance: MyConfig["createTextInstance"] = (
    content: string,
    rootContainerInstance: Container,
    hostContext: HostContext,
    internalInstanceHandle: OpaqueHandle
): TextInstance => {
    const text: TextNodeInstance = {
        tag: NodeType.text,
        text: content,
        children: [],
    }

    return text
}

const appendInitialChild: MyConfig["appendInitialChild"] = (parentInstance, child) => {
    //   assert(
    //     parentInstance.tag === NodeType.view,
    //     "cannot add children to text yet"
    //   );
    //   assert(child.tag !== "lingering text", "no free form text yet");
    parentInstance.children.push(child)
}

const finalizeInitialChildren: MyConfig["finalizeInitialChildren"] = (
    _parentInstance,
    _type,
    _props,
    _rootContainerInstance,
    _hostContext
) => {
    return false
}

const prepareUpdate: MyConfig["prepareUpdate"] = (
    instance,
    type,
    oldProps,
    newProps,
    rootContainerInstance,
    hostContext
) => {
    // null if nothing changed
    // here we will return a non null object to always commit a diff
    // TODO actually check that per node type
    return { old: oldProps, new: newProps }
}

const noTimeout = -1
const scheduleTimeout: MyConfig["scheduleTimeout"] = setTimeout
const cancelTimeout: MyConfig["cancelTimeout"] = clearTimeout

// --------- Mutation ----------------
const appendChild: MyConfig["appendChild"] = (parentInstance, child) => {
    console.log("appendChild")
    parentInstance.children.push(child)
}

const appendChildToContainer: MyConfig["appendChildToContainer"] = (container, child) => {
    //   assert(child.tag !== "lingering text", "no free form text yet");
    container.children.push(child)
    console.log("appendChildToContainer done", container)
}

const commitUpdate: MyConfig["commitUpdate"] = (
    instance,
    updatePayload,
    type,
    oldProps,
    newProps,
    internalInstanceHandle
) => {
    // copy pasted from create instance
    switch (type) {
        case NodeType.btn: {
            // TODO check props
            const { onClick } = newProps as ButtonProps
            assert(instance.tag === NodeType.btn)
            instance.onClick = onClick
            break
        }
        case NodeType.view: {
            // TODO check props
            const { border } = newProps as ViewProps
            assert(instance.tag === NodeType.view)
            instance.border = border
            break
        }
        case NodeType.text: {
            // TODO check props
            const { children, color, type: textType } = newProps as TextProps

            assert(instance.tag === NodeType.text)

            if (typeof children === "string" || typeof children === "number") {
                instance.text = children
            }

            instance.color = color
            instance.type = textType
            break
        }
    }
}

const commitTextUpdate: MyConfig["commitTextUpdate"] = (textInstance, oldText, newText) => {
    textInstance.text = newText
}

const removeChild: MyConfig["removeChild"] = (parentInstance, child) => {
    //   assert(
    //     parentInstance.tag === NodeType.view,
    //     "cannot add children to text yet"
    //   );
    //   assert(child.tag !== "lingering text", "no free form text yet");
    parentInstance.children = parentInstance.children.filter((c) => c !== child)
}

const removeChildFromContainer: MyConfig["removeChildFromContainer"] = (container, child) => {
    //   assert(child.tag !== "lingering text", "no free form text yet");
    container.children = container.children.filter((c) => c !== child)
}
// -------

const config: MyConfig = {
    // now,
    getPublicInstance,
    getRootHostContext,
    prepareForCommit,
    resetAfterCommit,
    getChildHostContext,
    shouldSetTextContent,
    createInstance,
    createTextInstance,
    appendChild,
    appendInitialChild,
    appendChildToContainer,
    finalizeInitialChildren,
    prepareUpdate,
    commitUpdate,
    commitTextUpdate,
    removeChild,
    removeChildFromContainer,
    isPrimaryRenderer: true,
    supportsMutation: true,
    supportsHydration: false,
    supportsPersistence: false,
    cancelTimeout,
    scheduleTimeout,
    noTimeout,
    getCurrentEventPriority: () => 1,
    clearContainer(container) {
        console.log("clearContainer")
    },
    beforeActiveInstanceBlur(): void {
        throw new Error("Function not implemented.")
    },
    afterActiveInstanceBlur(): void {
        throw new Error("Function not implemented.")
    },
    prepareScopeUpdate(scopeInstance, instance): void {
        throw new Error("Function not implemented.")
    },
    getInstanceFromScope(scopeInstance): Instance | null {
        throw new Error("Function not implemented.")
    },
    detachDeletedInstance: function (node: Instance): void {
        console.log("detachDeletedInstance")
        // noop
    },
    preparePortalMount(containerInfo) {
        // noop
        console.log("preparePortalMount", containerInfo)
    },
    getInstanceFromNode: function (node: any): ReactReconciler.Fiber | null | undefined {
        throw new Error("Function not implemented.")
    },
}

function onRecoverableError(error: Error) {
    // TODO: Turn this on once tests are fixed
    console.error(error)
}

const reconciler = ReactReconciler(config)

export const WorkerReconciler = {
    createPortal: (node: React.ReactNode, host: HostContext, key: string | null = null) => {
        const container: Container = {
            hostContext: host,
            children: [],
        }

        return reconciler.createPortal(node, container, null, key)
    },
    render: (node: React.ReactNode, host: HostContext, callback?: () => void): Container => {
        const container: Container = {
            hostContext: host,
            children: [],
        }

        // First clear any existing content.
        // clearContainer(container)

        const rootFiber = reconciler.createContainer(container, 0, null, false, false, "", onRecoverableError, null)

        reconciler.updateContainer(node, rootFiber, null, callback)

        return container
    },
}

// Reconciler.injectIntoDevTools({
//   bundleType: process.env.NODE_ENV === "production" ? 0 : 1,
//   version: "0.1.0",
//   rendererPackageName: "react-like",
// });
