/**
 * This is used for refs on host components.
 */
class UIHostComponent {
    _nativeTag: number

    constructor(tag: number) {
        this._nativeTag = tag
    }
}
