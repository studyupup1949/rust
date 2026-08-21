let wasm_bindgen = (function(exports) {
    let script_src;
    if (typeof document !== 'undefined' && document.currentScript !== null) {
        script_src = new URL(document.currentScript.src, location.href).toString();
    }

    /**
     * Output of [`verify_and_extract_actr_package`].
     *
     * Kept as an opaque handle exposed to JS via getters. Avoids round-tripping
     * binary bytes through JSON.
     */
    class ExtractedPackage {
        static __wrap(ptr) {
            const obj = Object.create(ExtractedPackage.prototype);
            obj.__wbg_ptr = ptr;
            ExtractedPackageFinalization.register(obj, obj.__wbg_ptr, obj);
            return obj;
        }
        __destroy_into_raw() {
            const ptr = this.__wbg_ptr;
            this.__wbg_ptr = 0;
            ExtractedPackageFinalization.unregister(this);
            return ptr;
        }
        free() {
            const ptr = this.__destroy_into_raw();
            wasm.__wbg_extractedpackage_free(ptr, 0);
        }
        /**
         * Verified binary bytes (WASM module) extracted from the `.actr` ZIP.
         * @returns {Uint8Array}
         */
        get binary() {
            const ret = wasm.extractedpackage_binary(this.__wbg_ptr);
            var v1 = getArrayU8FromWasm0(ret[0], ret[1]).slice();
            wasm.__wbindgen_free(ret[0], ret[1] * 1, 1);
            return v1;
        }
        /**
         * wasm-bindgen JS glue text extracted from `resources/*.js`, if any.
         * Returns `None` when the package carries no glue (guest-bridge mode or
         * pure-Rust packages).
         * @returns {string | undefined}
         */
        get glue_js() {
            const ret = wasm.extractedpackage_glue_js(this.__wbg_ptr);
            let v1;
            if (ret[0] !== 0) {
                v1 = getStringFromWasm0(ret[0], ret[1]).slice();
                wasm.__wbindgen_free(ret[0], ret[1] * 1, 1);
            }
            return v1;
        }
        /**
         * Verified package manifest, serialized as JSON. Fields mirror
         * `actr_pack::PackageManifest`.
         * @returns {string}
         */
        get manifest_json() {
            let deferred1_0;
            let deferred1_1;
            try {
                const ret = wasm.extractedpackage_manifest_json(this.__wbg_ptr);
                deferred1_0 = ret[0];
                deferred1_1 = ret[1];
                return getStringFromWasm0(ret[0], ret[1]);
            } finally {
                wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
            }
        }
    }
    if (Symbol.dispose) ExtractedPackage.prototype[Symbol.dispose] = ExtractedPackage.prototype.free;
    exports.ExtractedPackage = ExtractedPackage;

    /**
     * Handle an RPC control request originating from the DOM side.
     *
     * Message flow in unified-dispatcher mode:
     * - With `WORKLOAD`: `DOM -> workload.dispatch(route_key, payload, ctx) -> response`
     *   - Local route: the workload processes locally and may call remote targets via `ctx.call_raw()`
     *   - Remote route: the workload forwards to a remote actor via `ctx.call_raw()`
     * - Without `WORKLOAD`: `DOM -> HostGate -> Gate -> WebRTC`
     * @param {string} client_id
     * @param {any} payload
     * @returns {Promise<void>}
     */
    function handle_dom_control(client_id, payload) {
        const ptr0 = passStringToWasm0(client_id, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.handle_dom_control(ptr0, len0, payload);
        return ret;
    }
    exports.handle_dom_control = handle_dom_control;

    /**
     * @param {string} client_id
     * @param {any} payload
     */
    function handle_dom_fast_path(client_id, payload) {
        const ptr0 = passStringToWasm0(client_id, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.handle_dom_fast_path(ptr0, len0, payload);
        if (ret[1]) {
            throw takeFromExternrefTable0(ret[0]);
        }
    }
    exports.handle_dom_fast_path = handle_dom_fast_path;

    /**
     * @param {string} client_id
     * @param {any} payload
     * @returns {Promise<void>}
     */
    function handle_dom_webrtc_event(client_id, payload) {
        const ptr0 = passStringToWasm0(client_id, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.handle_dom_webrtc_event(ptr0, len0, payload);
        return ret;
    }
    exports.handle_dom_webrtc_event = handle_dom_webrtc_event;

    /**
     * WIT `host.call(target, route_key, payload) -> result<list<u8>, actr-error>`
     *
     * The web runtime only supports `dest::actor` for typed calls today (it has
     * no in-browser Shell/Local routing); other variants return
     * `not-implemented`. Keeps the WIT contract uniform between server and
     * browser — the variant arm exists, it just isn't wired.
     * @param {string} request_id
     * @param {any} target
     * @param {string} route_key
     * @param {Uint8Array} payload
     * @returns {Promise<Uint8Array>}
     */
    function host_call_async(request_id, target, route_key, payload) {
        const ptr0 = passStringToWasm0(request_id, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ptr1 = passStringToWasm0(route_key, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len1 = WASM_VECTOR_LEN;
        const ret = wasm.host_call_async(ptr0, len0, target, ptr1, len1, payload);
        return ret;
    }
    exports.host_call_async = host_call_async;

    /**
     * WIT `host.call-raw(target, route_key, payload) -> result<list<u8>, actr-error>`
     *
     * Async; returns a Promise that resolves to a `Uint8Array` on success or
     * rejects with a JS `Error` whose `actrErrorTag` names the WIT variant.
     *
     * The `request_id` first parameter identifies the owning dispatch and is
     * threaded through by the guest-side wrapper
     * (`actr_web_abi::guest::call_raw_with_request_id`). Two concurrent
     * dispatches no longer share a single thread-local context slot — they
     * resolve their respective `RuntimeContext` via `DISPATCH_CTXS`.
     * @param {string} request_id
     * @param {any} target
     * @param {string} route_key
     * @param {Uint8Array} payload
     * @returns {Promise<Uint8Array>}
     */
    function host_call_raw_async(request_id, target, route_key, payload) {
        const ptr0 = passStringToWasm0(request_id, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ptr1 = passStringToWasm0(route_key, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len1 = WASM_VECTOR_LEN;
        const ret = wasm.host_call_raw_async(ptr0, len0, target, ptr1, len1, payload);
        return ret;
    }
    exports.host_call_raw_async = host_call_raw_async;

    /**
     * WIT `host.discover(target_type) -> result<actr-id, actr-error>`.
     * @param {string} request_id
     * @param {any} target_type
     * @returns {Promise<any>}
     */
    function host_discover_async(request_id, target_type) {
        const ptr0 = passStringToWasm0(request_id, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.host_discover_async(ptr0, len0, target_type);
        return ret;
    }
    exports.host_discover_async = host_discover_async;

    /**
     * WIT `host.get-caller-id() -> option<actr-id>`. Returns `null` when the
     * host did not install a caller for this dispatch (lifecycle hooks).
     * @param {string} request_id
     * @returns {any}
     */
    function host_get_caller_id(request_id) {
        const ptr0 = passStringToWasm0(request_id, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.host_get_caller_id(ptr0, len0);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return takeFromExternrefTable0(ret[0]);
    }
    exports.host_get_caller_id = host_get_caller_id;

    /**
     * WIT `host.get-request-id() -> string`.
     *
     * Retaining the `request_id` input here is deliberate: the input and output
     * MUST match. It is asserted, giving us a cheap round-trip sanity check
     * between the guest-side wrapper (which has the request_id in hand from the
     * envelope) and the host-side dispatch table. The alternative — omitting
     * the parameter and treating it as a sentinel — would break uniformity
     * with the other 7 imports and require the WIT codegen to special-case it.
     * @param {string} request_id
     * @returns {string}
     */
    function host_get_request_id(request_id) {
        let deferred3_0;
        let deferred3_1;
        try {
            const ptr0 = passStringToWasm0(request_id, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len0 = WASM_VECTOR_LEN;
            const ret = wasm.host_get_request_id(ptr0, len0);
            var ptr2 = ret[0];
            var len2 = ret[1];
            if (ret[3]) {
                ptr2 = 0; len2 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred3_0 = ptr2;
            deferred3_1 = len2;
            return getStringFromWasm0(ptr2, len2);
        } finally {
            wasm.__wbindgen_free(deferred3_0, deferred3_1, 1);
        }
    }
    exports.host_get_request_id = host_get_request_id;

    /**
     * WIT `host.get-self-id() -> actr-id`.
     * @param {string} request_id
     * @returns {any}
     */
    function host_get_self_id(request_id) {
        const ptr0 = passStringToWasm0(request_id, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.host_get_self_id(ptr0, len0);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return takeFromExternrefTable0(ret[0]);
    }
    exports.host_get_self_id = host_get_self_id;

    /**
     * WIT `host.log-message(level, message)`.
     *
     * Maps to `log` crate levels. Levels outside the `trace/debug/info/warn/error`
     * set silently fall through to `info`. The `request_id` parameter is carried
     * for uniformity with the other host imports (and to annotate the log line);
     * it does not gate execution — logging from unknown dispatches still
     * surfaces.
     * @param {string} request_id
     * @param {string} level
     * @param {string} message
     */
    function host_log_message(request_id, level, message) {
        const ptr0 = passStringToWasm0(request_id, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ptr1 = passStringToWasm0(level, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len1 = WASM_VECTOR_LEN;
        const ptr2 = passStringToWasm0(message, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len2 = WASM_VECTOR_LEN;
        wasm.host_log_message(ptr0, len0, ptr1, len1, ptr2, len2);
    }
    exports.host_log_message = host_log_message;

    /**
     * WIT `host.tell(target, route_key, payload) -> result<_, actr-error>`.
     *
     * Fire-and-forget semantics. The web runtime maps this to `call_raw` with
     * `timeout_ms=0`; the result is discarded. Only `Dest::Actor` is wired.
     * @param {string} request_id
     * @param {any} target
     * @param {string} route_key
     * @param {Uint8Array} payload
     * @returns {Promise<void>}
     */
    function host_tell_async(request_id, target, route_key, payload) {
        const ptr0 = passStringToWasm0(request_id, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ptr1 = passStringToWasm0(route_key, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len1 = WASM_VECTOR_LEN;
        const ret = wasm.host_tell_async(ptr0, len0, target, ptr1, len1, payload);
        return ret;
    }
    exports.host_tell_async = host_tell_async;

    function init_global() {
        const ret = wasm.init_global();
        if (ret[1]) {
            throw takeFromExternrefTable0(ret[0]);
        }
    }
    exports.init_global = init_global;

    /**
     * Register a new client (browser tab) with the SW runtime.
     *
     * Each call creates an independent runtime with its own signaling connection,
     * actor registration, and WebRTC state. This enables multiple browser tabs
     * to work simultaneously without interfering with each other.
     * @param {string} client_id
     * @param {any} config
     * @param {MessagePort} port
     * @returns {Promise<void>}
     */
    function register_client(client_id, config, port) {
        const ptr0 = passStringToWasm0(client_id, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.register_client(ptr0, len0, config, port);
        return ret;
    }
    exports.register_client = register_client;

    /**
     * Register a dedicated DataChannel `MessagePort` received from the DOM side.
     *
     * After the DOM creates the DataChannel bridge:
     * 1. DOM: `port1 <-> DataChannel` for bidirectional forwarding
     * 2. DOM: transfers `port2` to the SW via a transferable object
     * 3. SW: this function receives `port2`, builds `WebRtcConnection`, and injects it into `WirePool`
     *
     * After injection, `DestTransport` is awakened through `ReadyWatcher`, and
     * subsequent outbound traffic is sent zero-copy through `DataLane::PostMessage(port)`.
     * @param {string} client_id
     * @param {string} peer_id
     * @param {MessagePort} port
     * @returns {Promise<void>}
     */
    function register_datachannel_port(client_id, peer_id, port) {
        const ptr0 = passStringToWasm0(client_id, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ptr1 = passStringToWasm0(peer_id, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len1 = WASM_VECTOR_LEN;
        const ret = wasm.register_datachannel_port(ptr0, len0, ptr1, len1, port);
        return ret;
    }
    exports.register_datachannel_port = register_datachannel_port;

    /**
     * Register a wasm-bindgen guest workload.
     *
     * `dispatch_fn` is a JS callback that forwards to the guest module's
     * `dispatch` export (emitted by `actr-web-abi`'s `__actr_workload_dispatch`).
     * Its signature must match:
     *
     * ```text
     * async (envelope: RpcEnvelopeJs) => Uint8Array
     * ```
     *
     * where `RpcEnvelopeJs` is the camelCase record built by sw-host on the
     * inbound side: `{ requestId: string, routeKey: string, payload: Uint8Array }`.
     *
     * The JS side is responsible for:
     * 1. Instantiating the wasm-bindgen guest bundle (`<name>.wbg/guest.js` +
     *    `_bg.wasm`) emitted by `tools/wit-compile-web` for the generated
     *    `actr-web-abi` shim.
     * 2. Installing the `actrHost*` JS globals that the guest imports — they
     *    proxy onto the `host_*_async` / `host_*` wasm-bindgen exports from
     *    this crate (see `bindings/web/packages/web-sdk/src/actor.sw.js`).
     * 3. Passing `(envelope) => guestBindgen.dispatch(envelope)` here as
     *    `dispatch_fn`.
     *
     * When this function is invoked the runtime installs the `ServiceHandlerFn`
     * used by [`WasmWorkload`], which the inbound dispatcher drives.
     *
     * # Naming
     *
     * Pre-Phase-8 this was `register_component_workload`, when the SW also
     * supported a Component Model + `jco`-transpiled guest. With CM removed
     * (Option U §11), the WBG-only name is the accurate one.
     * @param {Function} dispatch_fn
     */
    function register_guest_workload(dispatch_fn) {
        wasm.register_guest_workload(dispatch_fn);
    }
    exports.register_guest_workload = register_guest_workload;

    /**
     * Unregister a client (browser tab) from the SW runtime.
     *
     * Closes the signaling WebSocket (so the signaling server removes
     * the actor from its ServiceRegistry) and removes the client context.
     * Background tasks (signaling relay, heartbeat) will naturally stop
     * when the signaling connection drops.
     * @param {string} client_id
     * @returns {Promise<void>}
     */
    function unregister_client(client_id) {
        const ptr0 = passStringToWasm0(client_id, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.unregister_client(ptr0, len0);
        return ret;
    }
    exports.unregister_client = unregister_client;

    /**
     * Verify a `.actr` package against the provided trust anchors and return
     * its extracted parts.
     *
     * Browser-side equivalent of the `Hyper::verify_package` → `load_binary`
     * step on native. Always runs the full signature + binary hash chain;
     * there is no "skip verify" escape hatch.
     *
     * # Parameters
     * - `package_bytes` — the raw `.actr` ZIP bytes
     * - `trust_anchors_json` — JSON array of `TrustAnchor` entries
     *   (shape matches `actr_config::TrustAnchor`). The SW honours the first
     *   usable `kind = "static"` entry; `kind = "registry"` entries cause a
     *   hard error until the SW learns to do async AIS lookups.
     *
     * # Errors
     * Raises a `JsError` with a descriptive message on:
     * - malformed trust config
     * - no usable static anchor (empty, missing `pubkey_b64`, or only `registry`)
     * - invalid / wrong-size public key
     * - signature mismatch, tampered binary, missing manifest, etc.
     * @param {Uint8Array} package_bytes
     * @param {string} trust_anchors_json
     * @returns {ExtractedPackage}
     */
    function verify_and_extract_actr_package(package_bytes, trust_anchors_json) {
        const ptr0 = passArray8ToWasm0(package_bytes, wasm.__wbindgen_malloc);
        const len0 = WASM_VECTOR_LEN;
        const ptr1 = passStringToWasm0(trust_anchors_json, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len1 = WASM_VECTOR_LEN;
        const ret = wasm.verify_and_extract_actr_package(ptr0, len0, ptr1, len1);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return ExtractedPackage.__wrap(ret[0]);
    }
    exports.verify_and_extract_actr_package = verify_and_extract_actr_package;
    function __wbg_get_imports() {
        const import0 = {
            __proto__: null,
            __wbg_Error_fdd633d4bb5dd76a: function(arg0, arg1) {
                const ret = Error(getStringFromWasm0(arg0, arg1));
                return ret;
            },
            __wbg_Number_c4bdf66bb78f7977: function(arg0) {
                const ret = Number(arg0);
                return ret;
            },
            __wbg_String_8564e559799eccda: function(arg0, arg1) {
                const ret = String(arg1);
                const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
                const len1 = WASM_VECTOR_LEN;
                getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
                getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
            },
            __wbg___wbindgen_bigint_get_as_i64_d9e915702856f831: function(arg0, arg1) {
                const v = arg1;
                const ret = typeof(v) === 'bigint' ? v : undefined;
                getDataViewMemory0().setBigInt64(arg0 + 8 * 1, isLikeNone(ret) ? BigInt(0) : ret, true);
                getDataViewMemory0().setInt32(arg0 + 4 * 0, !isLikeNone(ret), true);
            },
            __wbg___wbindgen_boolean_get_edaed31a367ce1bd: function(arg0) {
                const v = arg0;
                const ret = typeof(v) === 'boolean' ? v : undefined;
                return isLikeNone(ret) ? 0xFFFFFF : ret ? 1 : 0;
            },
            __wbg___wbindgen_debug_string_8a447059637473e2: function(arg0, arg1) {
                const ret = debugString(arg1);
                const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
                const len1 = WASM_VECTOR_LEN;
                getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
                getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
            },
            __wbg___wbindgen_in_4990f46af709e33c: function(arg0, arg1) {
                const ret = arg0 in arg1;
                return ret;
            },
            __wbg___wbindgen_is_bigint_90b5ccfe67c78460: function(arg0) {
                const ret = typeof(arg0) === 'bigint';
                return ret;
            },
            __wbg___wbindgen_is_function_acc5528be2b923f2: function(arg0) {
                const ret = typeof(arg0) === 'function';
                return ret;
            },
            __wbg___wbindgen_is_null_6d937fbfb6478470: function(arg0) {
                const ret = arg0 === null;
                return ret;
            },
            __wbg___wbindgen_is_object_0beba4a1980d3eea: function(arg0) {
                const val = arg0;
                const ret = typeof(val) === 'object' && val !== null;
                return ret;
            },
            __wbg___wbindgen_is_string_1fca8072260dd261: function(arg0) {
                const ret = typeof(arg0) === 'string';
                return ret;
            },
            __wbg___wbindgen_is_undefined_721f8decd50c87a3: function(arg0) {
                const ret = arg0 === undefined;
                return ret;
            },
            __wbg___wbindgen_jsval_eq_4e8c38722cb8ff51: function(arg0, arg1) {
                const ret = arg0 === arg1;
                return ret;
            },
            __wbg___wbindgen_jsval_loose_eq_4b9aba9e5b3c4582: function(arg0, arg1) {
                const ret = arg0 == arg1;
                return ret;
            },
            __wbg___wbindgen_number_get_1cc01dd708740256: function(arg0, arg1) {
                const obj = arg1;
                const ret = typeof(obj) === 'number' ? obj : undefined;
                getDataViewMemory0().setFloat64(arg0 + 8 * 1, isLikeNone(ret) ? 0 : ret, true);
                getDataViewMemory0().setInt32(arg0 + 4 * 0, !isLikeNone(ret), true);
            },
            __wbg___wbindgen_string_get_71bb4348194e31f0: function(arg0, arg1) {
                const obj = arg1;
                const ret = typeof(obj) === 'string' ? obj : undefined;
                var ptr1 = isLikeNone(ret) ? 0 : passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
                var len1 = WASM_VECTOR_LEN;
                getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
                getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
            },
            __wbg___wbindgen_throw_ea4887a5f8f9a9db: function(arg0, arg1) {
                throw new Error(getStringFromWasm0(arg0, arg1));
            },
            __wbg__wbg_cb_unref_33c39e13d73b25f6: function(arg0) {
                arg0._wbg_cb_unref();
            },
            __wbg_abort_6e6ea7d259504afc: function(arg0) {
                arg0.abort();
            },
            __wbg_addEventListener_ea90bc131475777e: function() { return handleError(function (arg0, arg1, arg2, arg3) {
                arg0.addEventListener(getStringFromWasm0(arg1, arg2), arg3);
            }, arguments); },
            __wbg_add_32bdef4978943721: function() { return handleError(function (arg0, arg1) {
                const ret = arg0.add(arg1);
                return ret;
            }, arguments); },
            __wbg_add_45f31300f94b9beb: function() { return handleError(function (arg0, arg1, arg2) {
                const ret = arg0.add(arg1, arg2);
                return ret;
            }, arguments); },
            __wbg_arrayBuffer_ff96d08b7b6be32e: function() { return handleError(function (arg0) {
                const ret = arg0.arrayBuffer();
                return ret;
            }, arguments); },
            __wbg_call_0e855b388e315e17: function() { return handleError(function (arg0, arg1, arg2, arg3) {
                const ret = arg0.call(arg1, arg2, arg3);
                return ret;
            }, arguments); },
            __wbg_call_5575218572ead796: function() { return handleError(function (arg0, arg1, arg2) {
                const ret = arg0.call(arg1, arg2);
                return ret;
            }, arguments); },
            __wbg_call_8e98ed2f3c86c4b5: function() { return handleError(function (arg0, arg1) {
                const ret = arg0.call(arg1);
                return ret;
            }, arguments); },
            __wbg_clearTimeout_113b1cde814ec762: function(arg0) {
                const ret = clearTimeout(arg0);
                return ret;
            },
            __wbg_clear_258101e403ad3d21: function() { return handleError(function (arg0) {
                const ret = arg0.clear();
                return ret;
            }, arguments); },
            __wbg_close_26aa343c0d729303: function() { return handleError(function (arg0) {
                arg0.close();
            }, arguments); },
            __wbg_close_683df4d283fa0d31: function(arg0) {
                arg0.close();
            },
            __wbg_code_dfec082b46a31c19: function(arg0) {
                const ret = arg0.code;
                return ret;
            },
            __wbg_createIndex_3d7e743a6b9e4f27: function() { return handleError(function (arg0, arg1, arg2, arg3) {
                const ret = arg0.createIndex(getStringFromWasm0(arg1, arg2), arg3);
                return ret;
            }, arguments); },
            __wbg_createIndex_9f945022d859c080: function() { return handleError(function (arg0, arg1, arg2, arg3, arg4) {
                const ret = arg0.createIndex(getStringFromWasm0(arg1, arg2), arg3, arg4);
                return ret;
            }, arguments); },
            __wbg_createObjectStore_8b41c8907c72da4a: function() { return handleError(function (arg0, arg1, arg2, arg3) {
                const ret = arg0.createObjectStore(getStringFromWasm0(arg1, arg2), arg3);
                return ret;
            }, arguments); },
            __wbg_data_4a7f1308dbd33a21: function(arg0) {
                const ret = arg0.data;
                return ret;
            },
            __wbg_data_8511565af53c06b9: function(arg0) {
                const ret = arg0.data;
                return ret;
            },
            __wbg_debug_7271beced8b71cd4: function(arg0, arg1, arg2, arg3) {
                console.debug(arg0, arg1, arg2, arg3);
            },
            __wbg_deleteIndex_02c086a4e9ffd2b9: function() { return handleError(function (arg0, arg1, arg2) {
                arg0.deleteIndex(getStringFromWasm0(arg1, arg2));
            }, arguments); },
            __wbg_deleteObjectStore_311d37680cb6ed26: function() { return handleError(function (arg0, arg1, arg2) {
                arg0.deleteObjectStore(getStringFromWasm0(arg1, arg2));
            }, arguments); },
            __wbg_delete_328deb3af889d6dd: function() { return handleError(function (arg0, arg1) {
                const ret = arg0.delete(arg1);
                return ret;
            }, arguments); },
            __wbg_done_b62d4a7d2286852a: function(arg0) {
                const ret = arg0.done;
                return ret;
            },
            __wbg_encodeURIComponent_963d3e9b36ef7fe1: function(arg0, arg1) {
                const ret = encodeURIComponent(getStringFromWasm0(arg0, arg1));
                return ret;
            },
            __wbg_entries_c261c3fa1f281256: function(arg0) {
                const ret = Object.entries(arg0);
                return ret;
            },
            __wbg_error_2730901eef46e484: function() { return handleError(function (arg0) {
                const ret = arg0.error;
                return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
            }, arguments); },
            __wbg_error_50f60c611a3dcf64: function(arg0, arg1, arg2, arg3) {
                console.error(arg0, arg1, arg2, arg3);
            },
            __wbg_error_933f449d72fef598: function(arg0) {
                console.error(arg0);
            },
            __wbg_error_a6fa202b58aa1cd3: function(arg0, arg1) {
                let deferred0_0;
                let deferred0_1;
                try {
                    deferred0_0 = arg0;
                    deferred0_1 = arg1;
                    console.error(getStringFromWasm0(arg0, arg1));
                } finally {
                    wasm.__wbindgen_free(deferred0_0, deferred0_1, 1);
                }
            },
            __wbg_error_ea7fb0ccebd757ac: function(arg0) {
                const ret = arg0.error;
                return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
            },
            __wbg_fetch_db87be8a748781a2: function(arg0, arg1) {
                const ret = arg0.fetch(arg1);
                return ret;
            },
            __wbg_getAll_52c6a42bb7a26848: function() { return handleError(function (arg0) {
                const ret = arg0.getAll();
                return ret;
            }, arguments); },
            __wbg_getAll_c15c71bd30f697be: function() { return handleError(function (arg0, arg1, arg2) {
                const ret = arg0.getAll(arg1, arg2 >>> 0);
                return ret;
            }, arguments); },
            __wbg_getAll_e581098f3a7221b4: function() { return handleError(function (arg0, arg1) {
                const ret = arg0.getAll(arg1);
                return ret;
            }, arguments); },
            __wbg_getKey_78082d073cebe578: function() { return handleError(function (arg0, arg1) {
                const ret = arg0.getKey(arg1);
                return ret;
            }, arguments); },
            __wbg_getRandomValues_a697888e9ba1eee3: function() { return handleError(function (arg0, arg1) {
                globalThis.crypto.getRandomValues(getArrayU8FromWasm0(arg0, arg1));
            }, arguments); },
            __wbg_get_197a3fe98f169e38: function(arg0, arg1) {
                const ret = arg0[arg1 >>> 0];
                return ret;
            },
            __wbg_get_6479f49426d5d5d7: function(arg0, arg1, arg2) {
                const ret = arg1[arg2 >>> 0];
                var ptr1 = isLikeNone(ret) ? 0 : passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
                var len1 = WASM_VECTOR_LEN;
                getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
                getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
            },
            __wbg_get_9a29be2cb383ed9a: function() { return handleError(function (arg0, arg1) {
                const ret = Reflect.get(arg0, arg1);
                return ret;
            }, arguments); },
            __wbg_get_cad4a11b4bdb46d8: function() { return handleError(function (arg0, arg1, arg2, arg3) {
                const ret = arg1.get(getStringFromWasm0(arg2, arg3));
                var ptr1 = isLikeNone(ret) ? 0 : passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
                var len1 = WASM_VECTOR_LEN;
                getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
                getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
            }, arguments); },
            __wbg_get_cc81af70c9fceaea: function() { return handleError(function (arg0, arg1) {
                const ret = arg0.get(arg1);
                return ret;
            }, arguments); },
            __wbg_get_dddb90ff5d27a080: function() { return handleError(function (arg0, arg1) {
                const ret = Reflect.get(arg0, arg1);
                return ret;
            }, arguments); },
            __wbg_get_unchecked_54a4374c38e08460: function(arg0, arg1) {
                const ret = arg0[arg1 >>> 0];
                return ret;
            },
            __wbg_get_with_ref_key_6412cf3094599694: function(arg0, arg1) {
                const ret = arg0[arg1];
                return ret;
            },
            __wbg_headers_d9123c649c85d441: function(arg0) {
                const ret = arg0.headers;
                return ret;
            },
            __wbg_indexNames_7e0551c61a44ef9b: function(arg0) {
                const ret = arg0.indexNames;
                return ret;
            },
            __wbg_index_b4c61ab3121c4d42: function() { return handleError(function (arg0, arg1, arg2) {
                const ret = arg0.index(getStringFromWasm0(arg1, arg2));
                return ret;
            }, arguments); },
            __wbg_info_a392cd5b7536cfb5: function(arg0, arg1, arg2, arg3) {
                console.info(arg0, arg1, arg2, arg3);
            },
            __wbg_instanceof_ArrayBuffer_2a7bb09fee70c2da: function(arg0) {
                let result;
                try {
                    result = arg0 instanceof ArrayBuffer;
                } catch (_) {
                    result = false;
                }
                const ret = result;
                return ret;
            },
            __wbg_instanceof_IdbDatabase_514495af00e5eab0: function(arg0) {
                let result;
                try {
                    result = arg0 instanceof IDBDatabase;
                } catch (_) {
                    result = false;
                }
                const ret = result;
                return ret;
            },
            __wbg_instanceof_IdbFactory_54305bc2aeb37d94: function(arg0) {
                let result;
                try {
                    result = arg0 instanceof IDBFactory;
                } catch (_) {
                    result = false;
                }
                const ret = result;
                return ret;
            },
            __wbg_instanceof_IdbOpenDbRequest_2ecaf81349e1c7c2: function(arg0) {
                let result;
                try {
                    result = arg0 instanceof IDBOpenDBRequest;
                } catch (_) {
                    result = false;
                }
                const ret = result;
                return ret;
            },
            __wbg_instanceof_IdbRequest_a93f4449fab02673: function(arg0) {
                let result;
                try {
                    result = arg0 instanceof IDBRequest;
                } catch (_) {
                    result = false;
                }
                const ret = result;
                return ret;
            },
            __wbg_instanceof_IdbTransaction_69dbd37a47db2d17: function(arg0) {
                let result;
                try {
                    result = arg0 instanceof IDBTransaction;
                } catch (_) {
                    result = false;
                }
                const ret = result;
                return ret;
            },
            __wbg_instanceof_Map_afa18d5840c04c15: function(arg0) {
                let result;
                try {
                    result = arg0 instanceof Map;
                } catch (_) {
                    result = false;
                }
                const ret = result;
                return ret;
            },
            __wbg_instanceof_Object_60be3eaa7a661141: function(arg0) {
                let result;
                try {
                    result = arg0 instanceof Object;
                } catch (_) {
                    result = false;
                }
                const ret = result;
                return ret;
            },
            __wbg_instanceof_Promise_4614a0df6220bf3f: function(arg0) {
                let result;
                try {
                    result = arg0 instanceof Promise;
                } catch (_) {
                    result = false;
                }
                const ret = result;
                return ret;
            },
            __wbg_instanceof_Response_79948c98d1d2ba75: function(arg0) {
                let result;
                try {
                    result = arg0 instanceof Response;
                } catch (_) {
                    result = false;
                }
                const ret = result;
                return ret;
            },
            __wbg_instanceof_ServiceWorkerGlobalScope_f12d4d27c1e36d8d: function(arg0) {
                let result;
                try {
                    result = arg0 instanceof ServiceWorkerGlobalScope;
                } catch (_) {
                    result = false;
                }
                const ret = result;
                return ret;
            },
            __wbg_instanceof_Uint8Array_f080092dc70f5d58: function(arg0) {
                let result;
                try {
                    result = arg0 instanceof Uint8Array;
                } catch (_) {
                    result = false;
                }
                const ret = result;
                return ret;
            },
            __wbg_isArray_145a34fd0a38d37b: function(arg0) {
                const ret = Array.isArray(arg0);
                return ret;
            },
            __wbg_isSafeInteger_a3389a198582f5f6: function(arg0) {
                const ret = Number.isSafeInteger(arg0);
                return ret;
            },
            __wbg_iterator_cc47ba25a2be735a: function() {
                const ret = Symbol.iterator;
                return ret;
            },
            __wbg_keyPath_f740a596a0e2676b: function() { return handleError(function (arg0) {
                const ret = arg0.keyPath;
                return ret;
            }, arguments); },
            __wbg_length_589238bdcf171f0e: function(arg0) {
                const ret = arg0.length;
                return ret;
            },
            __wbg_length_7227514fa8c05fd0: function(arg0) {
                const ret = arg0.length;
                return ret;
            },
            __wbg_length_c6054974c0a6cdb9: function(arg0) {
                const ret = arg0.length;
                return ret;
            },
            __wbg_log_6b5af08dd293697f: function(arg0) {
                console.log(arg0);
            },
            __wbg_log_d282446d03691e72: function(arg0, arg1, arg2, arg3) {
                console.log(arg0, arg1, arg2, arg3);
            },
            __wbg_multiEntry_eef1bb8b41a1b3ea: function(arg0) {
                const ret = arg0.multiEntry;
                return ret;
            },
            __wbg_new_10e2f2ad134f940f: function() { return handleError(function () {
                const ret = new Headers();
                return ret;
            }, arguments); },
            __wbg_new_227d7c05414eb861: function() {
                const ret = new Error();
                return ret;
            },
            __wbg_new_2e117a478906f062: function() {
                const ret = new Object();
                return ret;
            },
            __wbg_new_36e147a8ced3c6e0: function() {
                const ret = new Array();
                return ret;
            },
            __wbg_new_51233fa2a760b272: function() { return handleError(function () {
                const ret = new AbortController();
                return ret;
            }, arguments); },
            __wbg_new_5a19eef57e9178b5: function() { return handleError(function (arg0, arg1) {
                const ret = new WebSocket(getStringFromWasm0(arg0, arg1));
                return ret;
            }, arguments); },
            __wbg_new_81880fb5002cb255: function(arg0) {
                const ret = new Uint8Array(arg0);
                return ret;
            },
            __wbg_new_e66a4b7758dd2e5c: function(arg0, arg1) {
                const ret = new Error(getStringFromWasm0(arg0, arg1));
                return ret;
            },
            __wbg_new_from_slice_543b875b27789a8f: function(arg0, arg1) {
                const ret = new Uint8Array(getArrayU8FromWasm0(arg0, arg1));
                return ret;
            },
            __wbg_new_typed_00a409eb4ec4f2d9: function(arg0, arg1) {
                try {
                    var state0 = {a: arg0, b: arg1};
                    var cb0 = (arg0, arg1) => {
                        const a = state0.a;
                        state0.a = 0;
                        try {
                            return wasm_bindgen__convert__closures_____invoke__h27cb4fce68c4daf0(a, state0.b, arg0, arg1);
                        } finally {
                            state0.a = a;
                        }
                    };
                    const ret = new Promise(cb0);
                    return ret;
                } finally {
                    state0.a = 0;
                }
            },
            __wbg_new_with_length_9b650f44b5c44a4e: function(arg0) {
                const ret = new Uint8Array(arg0 >>> 0);
                return ret;
            },
            __wbg_new_with_str_and_init_5b299538bdeeec64: function() { return handleError(function (arg0, arg1, arg2) {
                const ret = new Request(getStringFromWasm0(arg0, arg1), arg2);
                return ret;
            }, arguments); },
            __wbg_next_0c4066e251d2eff9: function() { return handleError(function (arg0) {
                const ret = arg0.next();
                return ret;
            }, arguments); },
            __wbg_next_402fa10b59ab20c3: function(arg0) {
                const ret = arg0.next;
                return ret;
            },
            __wbg_now_d2e0afbad4edbe82: function() {
                const ret = Date.now();
                return ret;
            },
            __wbg_objectStoreNames_1b3537217ff11c20: function(arg0) {
                const ret = arg0.objectStoreNames;
                return ret;
            },
            __wbg_objectStore_3468a23e50c7e125: function() { return handleError(function (arg0, arg1, arg2) {
                const ret = arg0.objectStore(getStringFromWasm0(arg1, arg2));
                return ret;
            }, arguments); },
            __wbg_open_07d64a5e2d11e344: function() { return handleError(function (arg0, arg1, arg2, arg3) {
                const ret = arg0.open(getStringFromWasm0(arg1, arg2), arg3 >>> 0);
                return ret;
            }, arguments); },
            __wbg_open_cd413714bc9879e4: function() { return handleError(function (arg0, arg1, arg2) {
                const ret = arg0.open(getStringFromWasm0(arg1, arg2));
                return ret;
            }, arguments); },
            __wbg_postMessage_5b1dc9f8de88488a: function() { return handleError(function (arg0, arg1) {
                arg0.postMessage(arg1);
            }, arguments); },
            __wbg_prototypesetcall_d721637c7ca66eb8: function(arg0, arg1, arg2) {
                Uint8Array.prototype.set.call(getArrayU8FromWasm0(arg0, arg1), arg2);
            },
            __wbg_push_f724b5db8acf89d2: function(arg0, arg1) {
                const ret = arg0.push(arg1);
                return ret;
            },
            __wbg_put_738d34320465aaf3: function() { return handleError(function (arg0, arg1, arg2) {
                const ret = arg0.put(arg1, arg2);
                return ret;
            }, arguments); },
            __wbg_put_8566cdc264332219: function() { return handleError(function (arg0, arg1) {
                const ret = arg0.put(arg1);
                return ret;
            }, arguments); },
            __wbg_queueMicrotask_1c9b3800e321a967: function(arg0) {
                const ret = arg0.queueMicrotask;
                return ret;
            },
            __wbg_queueMicrotask_311744e534a929a3: function(arg0) {
                queueMicrotask(arg0);
            },
            __wbg_random_3182549db57fb083: function() {
                const ret = Math.random();
                return ret;
            },
            __wbg_readyState_97951098f8995393: function(arg0) {
                const ret = arg0.readyState;
                return ret;
            },
            __wbg_reason_1f2868a7239f657b: function(arg0, arg1) {
                const ret = arg1.reason;
                const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
                const len1 = WASM_VECTOR_LEN;
                getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
                getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
            },
            __wbg_resolve_d82363d90af6928a: function(arg0) {
                const ret = Promise.resolve(arg0);
                return ret;
            },
            __wbg_result_ad4d0eede558cd6c: function() { return handleError(function (arg0) {
                const ret = arg0.result;
                return ret;
            }, arguments); },
            __wbg_send_982c819b9a1b34a5: function() { return handleError(function (arg0, arg1, arg2) {
                arg0.send(getArrayU8FromWasm0(arg1, arg2));
            }, arguments); },
            __wbg_setTimeout_ef24d2fc3ad97385: function() { return handleError(function (arg0, arg1) {
                const ret = setTimeout(arg0, arg1);
                return ret;
            }, arguments); },
            __wbg_set_0bf1fca872bc6d18: function(arg0, arg1, arg2) {
                arg0.set(getArrayU8FromWasm0(arg1, arg2));
            },
            __wbg_set_1c87dcfd4a93c514: function() { return handleError(function (arg0, arg1, arg2, arg3, arg4) {
                arg0.set(getStringFromWasm0(arg1, arg2), getStringFromWasm0(arg3, arg4));
            }, arguments); },
            __wbg_set_4564f7dc44fcb0c9: function() { return handleError(function (arg0, arg1, arg2) {
                const ret = Reflect.set(arg0, arg1, arg2);
                return ret;
            }, arguments); },
            __wbg_set_6be42768c690e380: function(arg0, arg1, arg2) {
                arg0[arg1] = arg2;
            },
            __wbg_set_auto_increment_5a55ad6938b60e2c: function(arg0, arg1) {
                arg0.autoIncrement = arg1 !== 0;
            },
            __wbg_set_binaryType_148427b11a8e6551: function(arg0, arg1) {
                arg0.binaryType = __wbindgen_enum_BinaryType[arg1];
            },
            __wbg_set_body_97c25d1c0051cb04: function(arg0, arg1) {
                arg0.body = arg1;
            },
            __wbg_set_dc601f4a69da0bc2: function(arg0, arg1, arg2) {
                arg0[arg1 >>> 0] = arg2;
            },
            __wbg_set_headers_6751c09a8e579ff7: function(arg0, arg1) {
                arg0.headers = arg1;
            },
            __wbg_set_key_path_c5ba68379d905ec6: function(arg0, arg1) {
                arg0.keyPath = arg1;
            },
            __wbg_set_method_1120482abe0934aa: function(arg0, arg1, arg2) {
                arg0.method = getStringFromWasm0(arg1, arg2);
            },
            __wbg_set_multi_entry_e747900162deecca: function(arg0, arg1) {
                arg0.multiEntry = arg1 !== 0;
            },
            __wbg_set_name_8130cd4ec4da32f0: function(arg0, arg1, arg2) {
                arg0.name = getStringFromWasm0(arg1, arg2);
            },
            __wbg_set_onabort_aa769067996be236: function(arg0, arg1) {
                arg0.onabort = arg1;
            },
            __wbg_set_onclose_8134952b2a9ec104: function(arg0, arg1) {
                arg0.onclose = arg1;
            },
            __wbg_set_oncomplete_56fb9939584534f1: function(arg0, arg1) {
                arg0.oncomplete = arg1;
            },
            __wbg_set_onerror_0803e0826d3abdc4: function(arg0, arg1) {
                arg0.onerror = arg1;
            },
            __wbg_set_onerror_3f68563f77d362f1: function(arg0, arg1) {
                arg0.onerror = arg1;
            },
            __wbg_set_onerror_ba8db3530f46f30a: function(arg0, arg1) {
                arg0.onerror = arg1;
            },
            __wbg_set_onmessage_397a79f643011142: function(arg0, arg1) {
                arg0.onmessage = arg1;
            },
            __wbg_set_onopen_ca8d311fe5282041: function(arg0, arg1) {
                arg0.onopen = arg1;
            },
            __wbg_set_onsuccess_2cdfe0be022e28fa: function(arg0, arg1) {
                arg0.onsuccess = arg1;
            },
            __wbg_set_onupgradeneeded_b840f47064664247: function(arg0, arg1) {
                arg0.onupgradeneeded = arg1;
            },
            __wbg_set_onversionchange_9381fdf7f68c845f: function(arg0, arg1) {
                arg0.onversionchange = arg1;
            },
            __wbg_set_signal_4a69430cb12800f3: function(arg0, arg1) {
                arg0.signal = arg1;
            },
            __wbg_set_unique_88fcfd491d8e7548: function(arg0, arg1) {
                arg0.unique = arg1 !== 0;
            },
            __wbg_signal_4d9d567be73ea52c: function(arg0) {
                const ret = arg0.signal;
                return ret;
            },
            __wbg_stack_3b0d974bbf31e44f: function(arg0, arg1) {
                const ret = arg1.stack;
                const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
                const len1 = WASM_VECTOR_LEN;
                getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
                getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
            },
            __wbg_static_accessor_GLOBAL_THIS_2fee5048bcca5938: function() {
                const ret = typeof globalThis === 'undefined' ? null : globalThis;
                return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
            },
            __wbg_static_accessor_GLOBAL_ce44e66a4935da8c: function() {
                const ret = typeof global === 'undefined' ? null : global;
                return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
            },
            __wbg_static_accessor_SELF_44f6e0cb5e67cdad: function() {
                const ret = typeof self === 'undefined' ? null : self;
                return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
            },
            __wbg_static_accessor_WINDOW_168f178805d978fe: function() {
                const ret = typeof window === 'undefined' ? null : window;
                return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
            },
            __wbg_statusText_0f412961834ccfe9: function(arg0, arg1) {
                const ret = arg1.statusText;
                const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
                const len1 = WASM_VECTOR_LEN;
                getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
                getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
            },
            __wbg_status_0053aa6239760447: function(arg0) {
                const ret = arg0.status;
                return ret;
            },
            __wbg_target_4387d5c508f1ecbd: function(arg0) {
                const ret = arg0.target;
                return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
            },
            __wbg_then_05edfc8a4fea5106: function(arg0, arg1, arg2) {
                const ret = arg0.then(arg1, arg2);
                return ret;
            },
            __wbg_then_591b6b3a75ee817a: function(arg0, arg1) {
                const ret = arg0.then(arg1);
                return ret;
            },
            __wbg_transaction_52047bb6fef2c278: function() { return handleError(function (arg0, arg1, arg2) {
                const ret = arg0.transaction(arg1, __wbindgen_enum_IdbTransactionMode[arg2]);
                return ret;
            }, arguments); },
            __wbg_transaction_deacadb8e213940d: function(arg0) {
                const ret = arg0.transaction;
                return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
            },
            __wbg_unique_11e62e0ca08852d9: function(arg0) {
                const ret = arg0.unique;
                return ret;
            },
            __wbg_url_94ca60bf2fbfc218: function(arg0, arg1) {
                const ret = arg1.url;
                const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
                const len1 = WASM_VECTOR_LEN;
                getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
                getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
            },
            __wbg_value_49f783bb59765962: function(arg0) {
                const ret = arg0.value;
                return ret;
            },
            __wbg_warn_88c4a5bd9a322000: function(arg0, arg1, arg2, arg3) {
                console.warn(arg0, arg1, arg2, arg3);
            },
            __wbg_wasClean_4e4eb8356fe0302e: function(arg0) {
                const ret = arg0.wasClean;
                return ret;
            },
            __wbindgen_cast_0000000000000001: function(arg0, arg1) {
                // Cast intrinsic for `Closure(Closure { owned: true, function: Function { arguments: [Externref], shim_idx: 4, ret: Unit, inner_ret: Some(Unit) }, mutable: true }) -> Externref`.
                const ret = makeMutClosure(arg0, arg1, wasm_bindgen__convert__closures_____invoke__h12d36a422a1065a2);
                return ret;
            },
            __wbindgen_cast_0000000000000002: function(arg0, arg1) {
                // Cast intrinsic for `Closure(Closure { owned: true, function: Function { arguments: [Externref], shim_idx: 621, ret: Result(Unit), inner_ret: Some(Result(Unit)) }, mutable: true }) -> Externref`.
                const ret = makeMutClosure(arg0, arg1, wasm_bindgen__convert__closures_____invoke__hf59e57ae18704e9d);
                return ret;
            },
            __wbindgen_cast_0000000000000003: function(arg0, arg1) {
                // Cast intrinsic for `Closure(Closure { owned: true, function: Function { arguments: [NamedExternref("CloseEvent")], shim_idx: 4, ret: Unit, inner_ret: Some(Unit) }, mutable: true }) -> Externref`.
                const ret = makeMutClosure(arg0, arg1, wasm_bindgen__convert__closures_____invoke__h12d36a422a1065a2_2);
                return ret;
            },
            __wbindgen_cast_0000000000000004: function(arg0, arg1) {
                // Cast intrinsic for `Closure(Closure { owned: true, function: Function { arguments: [NamedExternref("Event")], shim_idx: 559, ret: Unit, inner_ret: Some(Unit) }, mutable: true }) -> Externref`.
                const ret = makeMutClosure(arg0, arg1, wasm_bindgen__convert__closures_____invoke__h45d9549bec99e086);
                return ret;
            },
            __wbindgen_cast_0000000000000005: function(arg0, arg1) {
                // Cast intrinsic for `Closure(Closure { owned: true, function: Function { arguments: [NamedExternref("ExtendableMessageEvent")], shim_idx: 4, ret: Unit, inner_ret: Some(Unit) }, mutable: true }) -> Externref`.
                const ret = makeMutClosure(arg0, arg1, wasm_bindgen__convert__closures_____invoke__h12d36a422a1065a2_4);
                return ret;
            },
            __wbindgen_cast_0000000000000006: function(arg0, arg1) {
                // Cast intrinsic for `Closure(Closure { owned: true, function: Function { arguments: [NamedExternref("IDBVersionChangeEvent")], shim_idx: 311, ret: Unit, inner_ret: Some(Unit) }, mutable: true }) -> Externref`.
                const ret = makeMutClosure(arg0, arg1, wasm_bindgen__convert__closures_____invoke__hfbc01e03b65f0d81);
                return ret;
            },
            __wbindgen_cast_0000000000000007: function(arg0, arg1) {
                // Cast intrinsic for `Closure(Closure { owned: true, function: Function { arguments: [NamedExternref("MessageEvent")], shim_idx: 4, ret: Unit, inner_ret: Some(Unit) }, mutable: true }) -> Externref`.
                const ret = makeMutClosure(arg0, arg1, wasm_bindgen__convert__closures_____invoke__h12d36a422a1065a2_6);
                return ret;
            },
            __wbindgen_cast_0000000000000008: function(arg0, arg1) {
                // Cast intrinsic for `Closure(Closure { owned: true, function: Function { arguments: [], shim_idx: 363, ret: Unit, inner_ret: Some(Unit) }, mutable: true }) -> Externref`.
                const ret = makeMutClosure(arg0, arg1, wasm_bindgen__convert__closures_____invoke__h1233b70bbe07723a);
                return ret;
            },
            __wbindgen_cast_0000000000000009: function(arg0) {
                // Cast intrinsic for `F64 -> Externref`.
                const ret = arg0;
                return ret;
            },
            __wbindgen_cast_000000000000000a: function(arg0) {
                // Cast intrinsic for `I64 -> Externref`.
                const ret = arg0;
                return ret;
            },
            __wbindgen_cast_000000000000000b: function(arg0, arg1) {
                // Cast intrinsic for `Ref(Slice(U8)) -> NamedExternref("Uint8Array")`.
                const ret = getArrayU8FromWasm0(arg0, arg1);
                return ret;
            },
            __wbindgen_cast_000000000000000c: function(arg0, arg1) {
                // Cast intrinsic for `Ref(String) -> Externref`.
                const ret = getStringFromWasm0(arg0, arg1);
                return ret;
            },
            __wbindgen_cast_000000000000000d: function(arg0) {
                // Cast intrinsic for `U64 -> Externref`.
                const ret = BigInt.asUintN(64, arg0);
                return ret;
            },
            __wbindgen_init_externref_table: function() {
                const table = wasm.__wbindgen_externrefs;
                const offset = table.grow(4);
                table.set(0, undefined);
                table.set(offset + 0, undefined);
                table.set(offset + 1, null);
                table.set(offset + 2, true);
                table.set(offset + 3, false);
            },
        };
        return {
            __proto__: null,
            "./actr_sw_host_bg.js": import0,
        };
    }

    function wasm_bindgen__convert__closures_____invoke__h1233b70bbe07723a(arg0, arg1) {
        wasm.wasm_bindgen__convert__closures_____invoke__h1233b70bbe07723a(arg0, arg1);
    }

    function wasm_bindgen__convert__closures_____invoke__h12d36a422a1065a2(arg0, arg1, arg2) {
        wasm.wasm_bindgen__convert__closures_____invoke__h12d36a422a1065a2(arg0, arg1, arg2);
    }

    function wasm_bindgen__convert__closures_____invoke__h12d36a422a1065a2_2(arg0, arg1, arg2) {
        wasm.wasm_bindgen__convert__closures_____invoke__h12d36a422a1065a2_2(arg0, arg1, arg2);
    }

    function wasm_bindgen__convert__closures_____invoke__h45d9549bec99e086(arg0, arg1, arg2) {
        wasm.wasm_bindgen__convert__closures_____invoke__h45d9549bec99e086(arg0, arg1, arg2);
    }

    function wasm_bindgen__convert__closures_____invoke__h12d36a422a1065a2_4(arg0, arg1, arg2) {
        wasm.wasm_bindgen__convert__closures_____invoke__h12d36a422a1065a2_4(arg0, arg1, arg2);
    }

    function wasm_bindgen__convert__closures_____invoke__hfbc01e03b65f0d81(arg0, arg1, arg2) {
        wasm.wasm_bindgen__convert__closures_____invoke__hfbc01e03b65f0d81(arg0, arg1, arg2);
    }

    function wasm_bindgen__convert__closures_____invoke__h12d36a422a1065a2_6(arg0, arg1, arg2) {
        wasm.wasm_bindgen__convert__closures_____invoke__h12d36a422a1065a2_6(arg0, arg1, arg2);
    }

    function wasm_bindgen__convert__closures_____invoke__hf59e57ae18704e9d(arg0, arg1, arg2) {
        const ret = wasm.wasm_bindgen__convert__closures_____invoke__hf59e57ae18704e9d(arg0, arg1, arg2);
        if (ret[1]) {
            throw takeFromExternrefTable0(ret[0]);
        }
    }

    function wasm_bindgen__convert__closures_____invoke__h27cb4fce68c4daf0(arg0, arg1, arg2, arg3) {
        wasm.wasm_bindgen__convert__closures_____invoke__h27cb4fce68c4daf0(arg0, arg1, arg2, arg3);
    }


    const __wbindgen_enum_BinaryType = ["blob", "arraybuffer"];


    const __wbindgen_enum_IdbTransactionMode = ["readonly", "readwrite", "versionchange", "readwriteflush", "cleanup"];
    const ExtractedPackageFinalization = (typeof FinalizationRegistry === 'undefined')
        ? { register: () => {}, unregister: () => {} }
        : new FinalizationRegistry(ptr => wasm.__wbg_extractedpackage_free(ptr, 1));

    function addToExternrefTable0(obj) {
        const idx = wasm.__externref_table_alloc();
        wasm.__wbindgen_externrefs.set(idx, obj);
        return idx;
    }

    const CLOSURE_DTORS = (typeof FinalizationRegistry === 'undefined')
        ? { register: () => {}, unregister: () => {} }
        : new FinalizationRegistry(state => wasm.__wbindgen_destroy_closure(state.a, state.b));

    function debugString(val) {
        // primitive types
        const type = typeof val;
        if (type == 'number' || type == 'boolean' || val == null) {
            return  `${val}`;
        }
        if (type == 'string') {
            return `"${val}"`;
        }
        if (type == 'symbol') {
            const description = val.description;
            if (description == null) {
                return 'Symbol';
            } else {
                return `Symbol(${description})`;
            }
        }
        if (type == 'function') {
            const name = val.name;
            if (typeof name == 'string' && name.length > 0) {
                return `Function(${name})`;
            } else {
                return 'Function';
            }
        }
        // objects
        if (Array.isArray(val)) {
            const length = val.length;
            let debug = '[';
            if (length > 0) {
                debug += debugString(val[0]);
            }
            for(let i = 1; i < length; i++) {
                debug += ', ' + debugString(val[i]);
            }
            debug += ']';
            return debug;
        }
        // Test for built-in
        const builtInMatches = /\[object ([^\]]+)\]/.exec(toString.call(val));
        let className;
        if (builtInMatches && builtInMatches.length > 1) {
            className = builtInMatches[1];
        } else {
            // Failed to match the standard '[object ClassName]'
            return toString.call(val);
        }
        if (className == 'Object') {
            // we're a user defined class or Object
            // JSON.stringify avoids problems with cycles, and is generally much
            // easier than looping through ownProperties of `val`.
            try {
                return 'Object(' + JSON.stringify(val) + ')';
            } catch (_) {
                return 'Object';
            }
        }
        // errors
        if (val instanceof Error) {
            return `${val.name}: ${val.message}\n${val.stack}`;
        }
        // TODO we could test for more things here, like `Set`s and `Map`s.
        return className;
    }

    function getArrayU8FromWasm0(ptr, len) {
        ptr = ptr >>> 0;
        return getUint8ArrayMemory0().subarray(ptr / 1, ptr / 1 + len);
    }

    let cachedDataViewMemory0 = null;
    function getDataViewMemory0() {
        if (cachedDataViewMemory0 === null || cachedDataViewMemory0.buffer.detached === true || (cachedDataViewMemory0.buffer.detached === undefined && cachedDataViewMemory0.buffer !== wasm.memory.buffer)) {
            cachedDataViewMemory0 = new DataView(wasm.memory.buffer);
        }
        return cachedDataViewMemory0;
    }

    function getStringFromWasm0(ptr, len) {
        return decodeText(ptr >>> 0, len);
    }

    let cachedUint8ArrayMemory0 = null;
    function getUint8ArrayMemory0() {
        if (cachedUint8ArrayMemory0 === null || cachedUint8ArrayMemory0.byteLength === 0) {
            cachedUint8ArrayMemory0 = new Uint8Array(wasm.memory.buffer);
        }
        return cachedUint8ArrayMemory0;
    }

    function handleError(f, args) {
        try {
            return f.apply(this, args);
        } catch (e) {
            const idx = addToExternrefTable0(e);
            wasm.__wbindgen_exn_store(idx);
        }
    }

    function isLikeNone(x) {
        return x === undefined || x === null;
    }

    function makeMutClosure(arg0, arg1, f) {
        const state = { a: arg0, b: arg1, cnt: 1 };
        const real = (...args) => {

            // First up with a closure we increment the internal reference
            // count. This ensures that the Rust closure environment won't
            // be deallocated while we're invoking it.
            state.cnt++;
            const a = state.a;
            state.a = 0;
            try {
                return f(a, state.b, ...args);
            } finally {
                state.a = a;
                real._wbg_cb_unref();
            }
        };
        real._wbg_cb_unref = () => {
            if (--state.cnt === 0) {
                wasm.__wbindgen_destroy_closure(state.a, state.b);
                state.a = 0;
                CLOSURE_DTORS.unregister(state);
            }
        };
        CLOSURE_DTORS.register(real, state, state);
        return real;
    }

    function passArray8ToWasm0(arg, malloc) {
        const ptr = malloc(arg.length * 1, 1) >>> 0;
        getUint8ArrayMemory0().set(arg, ptr / 1);
        WASM_VECTOR_LEN = arg.length;
        return ptr;
    }

    function passStringToWasm0(arg, malloc, realloc) {
        if (realloc === undefined) {
            const buf = cachedTextEncoder.encode(arg);
            const ptr = malloc(buf.length, 1) >>> 0;
            getUint8ArrayMemory0().subarray(ptr, ptr + buf.length).set(buf);
            WASM_VECTOR_LEN = buf.length;
            return ptr;
        }

        let len = arg.length;
        let ptr = malloc(len, 1) >>> 0;

        const mem = getUint8ArrayMemory0();

        let offset = 0;

        for (; offset < len; offset++) {
            const code = arg.charCodeAt(offset);
            if (code > 0x7F) break;
            mem[ptr + offset] = code;
        }
        if (offset !== len) {
            if (offset !== 0) {
                arg = arg.slice(offset);
            }
            ptr = realloc(ptr, len, len = offset + arg.length * 3, 1) >>> 0;
            const view = getUint8ArrayMemory0().subarray(ptr + offset, ptr + len);
            const ret = cachedTextEncoder.encodeInto(arg, view);

            offset += ret.written;
            ptr = realloc(ptr, len, offset, 1) >>> 0;
        }

        WASM_VECTOR_LEN = offset;
        return ptr;
    }

    function takeFromExternrefTable0(idx) {
        const value = wasm.__wbindgen_externrefs.get(idx);
        wasm.__externref_table_dealloc(idx);
        return value;
    }

    let cachedTextDecoder = new TextDecoder('utf-8', { ignoreBOM: true, fatal: true });
    cachedTextDecoder.decode();
    function decodeText(ptr, len) {
        return cachedTextDecoder.decode(getUint8ArrayMemory0().subarray(ptr, ptr + len));
    }

    const cachedTextEncoder = new TextEncoder();

    if (!('encodeInto' in cachedTextEncoder)) {
        cachedTextEncoder.encodeInto = function (arg, view) {
            const buf = cachedTextEncoder.encode(arg);
            view.set(buf);
            return {
                read: arg.length,
                written: buf.length
            };
        };
    }

    let WASM_VECTOR_LEN = 0;

    let wasmModule, wasmInstance, wasm;
    function __wbg_finalize_init(instance, module) {
        wasmInstance = instance;
        wasm = instance.exports;
        wasmModule = module;
        cachedDataViewMemory0 = null;
        cachedUint8ArrayMemory0 = null;
        wasm.__wbindgen_start();
        return wasm;
    }

    async function __wbg_load(module, imports) {
        if (typeof Response === 'function' && module instanceof Response) {
            if (typeof WebAssembly.instantiateStreaming === 'function') {
                try {
                    return await WebAssembly.instantiateStreaming(module, imports);
                } catch (e) {
                    const validResponse = module.ok && expectedResponseType(module.type);

                    if (validResponse && module.headers.get('Content-Type') !== 'application/wasm') {
                        console.warn("`WebAssembly.instantiateStreaming` failed because your server does not serve Wasm with `application/wasm` MIME type. Falling back to `WebAssembly.instantiate` which is slower. Original error:\n", e);

                    } else { throw e; }
                }
            }

            const bytes = await module.arrayBuffer();
            return await WebAssembly.instantiate(bytes, imports);
        } else {
            const instance = await WebAssembly.instantiate(module, imports);

            if (instance instanceof WebAssembly.Instance) {
                return { instance, module };
            } else {
                return instance;
            }
        }

        function expectedResponseType(type) {
            switch (type) {
                case 'basic': case 'cors': case 'default': return true;
            }
            return false;
        }
    }

    function initSync(module) {
        if (wasm !== undefined) return wasm;


        if (module !== undefined) {
            if (Object.getPrototypeOf(module) === Object.prototype) {
                ({module} = module)
            } else {
                console.warn('using deprecated parameters for `initSync()`; pass a single object instead')
            }
        }

        const imports = __wbg_get_imports();
        if (!(module instanceof WebAssembly.Module)) {
            module = new WebAssembly.Module(module);
        }
        const instance = new WebAssembly.Instance(module, imports);
        return __wbg_finalize_init(instance, module);
    }

    async function __wbg_init(module_or_path) {
        if (wasm !== undefined) return wasm;


        if (module_or_path !== undefined) {
            if (Object.getPrototypeOf(module_or_path) === Object.prototype) {
                ({module_or_path} = module_or_path)
            } else {
                console.warn('using deprecated parameters for the initialization function; pass a single object instead')
            }
        }

        if (module_or_path === undefined && script_src !== undefined) {
            module_or_path = script_src.replace(/\.js$/, "_bg.wasm");
        }
        const imports = __wbg_get_imports();

        if (typeof module_or_path === 'string' || (typeof Request === 'function' && module_or_path instanceof Request) || (typeof URL === 'function' && module_or_path instanceof URL)) {
            module_or_path = fetch(module_or_path);
        }

        const { instance, module } = await __wbg_load(await module_or_path, imports);

        return __wbg_finalize_init(instance, module);
    }

    return Object.assign(__wbg_init, { initSync }, exports);
})({ __proto__: null });
