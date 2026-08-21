'use strict';

require('@smui/paper');
var Button = require('@smui/button');
require('@smui/textfield');
require('@smui/textfield/icon/index');
var svelteNotifications = require('@beyonk/svelte-notifications');
var detectEthereumProvider = require('@metamask/detect-provider');
require('web3');
var require$$0 = require('backo2');
var require$$4 = require('graphql/language/printer');
var require$$5 = require('graphql/utilities/getOperationAST');

/** @returns {void} */
function noop() {}

function run(fn) {
	return fn();
}

function blank_object() {
	return Object.create(null);
}

/**
 * @param {Function[]} fns
 * @returns {void}
 */
function run_all(fns) {
	fns.forEach(run);
}

/**
 * @param {any} thing
 * @returns {thing is Function}
 */
function is_function(thing) {
	return typeof thing === 'function';
}

/** @returns {boolean} */
function safe_not_equal(a, b) {
	return a != a ? b == b : a !== b || (a && typeof a === 'object') || typeof a === 'function';
}

/** @returns {boolean} */
function is_empty(obj) {
	return Object.keys(obj).length === 0;
}

/**
 * @param {Node} target
 * @param {Node} node
 * @returns {void}
 */
function append(target, node) {
	target.appendChild(node);
}

/**
 * @param {Node} target
 * @param {string} style_sheet_id
 * @param {string} styles
 * @returns {void}
 */
function append_styles(target, style_sheet_id, styles) {
	const append_styles_to = get_root_for_style(target);
	if (!append_styles_to.getElementById(style_sheet_id)) {
		const style = element('style');
		style.id = style_sheet_id;
		style.textContent = styles;
		append_stylesheet(append_styles_to, style);
	}
}

/**
 * @param {Node} node
 * @returns {ShadowRoot | Document}
 */
function get_root_for_style(node) {
	if (!node) return document;
	const root = node.getRootNode ? node.getRootNode() : node.ownerDocument;
	if (root && /** @type {ShadowRoot} */ (root).host) {
		return /** @type {ShadowRoot} */ (root);
	}
	return node.ownerDocument;
}

/**
 * @param {ShadowRoot | Document} node
 * @param {HTMLStyleElement} style
 * @returns {CSSStyleSheet}
 */
function append_stylesheet(node, style) {
	append(/** @type {Document} */ (node).head || node, style);
	return style.sheet;
}

/**
 * @param {Node} target
 * @param {Node} node
 * @param {Node} [anchor]
 * @returns {void}
 */
function insert(target, node, anchor) {
	target.insertBefore(node, anchor || null);
}

/**
 * @param {Node} node
 * @returns {void}
 */
function detach(node) {
	if (node.parentNode) {
		node.parentNode.removeChild(node);
	}
}

/**
 * @template {keyof HTMLElementTagNameMap} K
 * @param {K} name
 * @returns {HTMLElementTagNameMap[K]}
 */
function element(name) {
	return document.createElement(name);
}

/**
 * @param {string} data
 * @returns {Text}
 */
function text(data) {
	return document.createTextNode(data);
}

/**
 * @returns {Text} */
function space() {
	return text(' ');
}

/**
 * @returns {Text} */
function empty() {
	return text('');
}

/**
 * @param {EventTarget} node
 * @param {string} event
 * @param {EventListenerOrEventListenerObject} handler
 * @param {boolean | AddEventListenerOptions | EventListenerOptions} [options]
 * @returns {() => void}
 */
function listen(node, event, handler, options) {
	node.addEventListener(event, handler, options);
	return () => node.removeEventListener(event, handler, options);
}

/**
 * @param {Element} node
 * @param {string} attribute
 * @param {string} [value]
 * @returns {void}
 */
function attr(node, attribute, value) {
	if (value == null) node.removeAttribute(attribute);
	else if (node.getAttribute(attribute) !== value) node.setAttribute(attribute, value);
}

/**
 * @param {Element} element
 * @returns {ChildNode[]}
 */
function children(element) {
	return Array.from(element.childNodes);
}

/**
 * @returns {void} */
function set_input_value(input, value) {
	input.value = value == null ? '' : value;
}

/**
 * @typedef {Node & {
 * 	claim_order?: number;
 * 	hydrate_init?: true;
 * 	actual_end_child?: NodeEx;
 * 	childNodes: NodeListOf<NodeEx>;
 * }} NodeEx
 */

/** @typedef {ChildNode & NodeEx} ChildNodeEx */

/** @typedef {NodeEx & { claim_order: number }} NodeEx2 */

/**
 * @typedef {ChildNodeEx[] & {
 * 	claim_info?: {
 * 		last_index: number;
 * 		total_claimed: number;
 * 	};
 * }} ChildNodeArray
 */

let current_component;

/** @returns {void} */
function set_current_component(component) {
	current_component = component;
}

const dirty_components = [];
const binding_callbacks = [];

let render_callbacks = [];

const flush_callbacks = [];

const resolved_promise = /* @__PURE__ */ Promise.resolve();

let update_scheduled = false;

/** @returns {void} */
function schedule_update() {
	if (!update_scheduled) {
		update_scheduled = true;
		resolved_promise.then(flush);
	}
}

/** @returns {void} */
function add_render_callback(fn) {
	render_callbacks.push(fn);
}

// flush() calls callbacks in this order:
// 1. All beforeUpdate callbacks, in order: parents before children
// 2. All bind:this callbacks, in reverse order: children before parents.
// 3. All afterUpdate callbacks, in order: parents before children. EXCEPT
//    for afterUpdates called during the initial onMount, which are called in
//    reverse order: children before parents.
// Since callbacks might update component values, which could trigger another
// call to flush(), the following steps guard against this:
// 1. During beforeUpdate, any updated components will be added to the
//    dirty_components array and will cause a reentrant call to flush(). Because
//    the flush index is kept outside the function, the reentrant call will pick
//    up where the earlier call left off and go through all dirty components. The
//    current_component value is saved and restored so that the reentrant call will
//    not interfere with the "parent" flush() call.
// 2. bind:this callbacks cannot trigger new flush() calls.
// 3. During afterUpdate, any updated components will NOT have their afterUpdate
//    callback called a second time; the seen_callbacks set, outside the flush()
//    function, guarantees this behavior.
const seen_callbacks = new Set();

let flushidx = 0; // Do *not* move this inside the flush() function

/** @returns {void} */
function flush() {
	// Do not reenter flush while dirty components are updated, as this can
	// result in an infinite loop. Instead, let the inner flush handle it.
	// Reentrancy is ok afterwards for bindings etc.
	if (flushidx !== 0) {
		return;
	}
	const saved_component = current_component;
	do {
		// first, call beforeUpdate functions
		// and update components
		try {
			while (flushidx < dirty_components.length) {
				const component = dirty_components[flushidx];
				flushidx++;
				set_current_component(component);
				update(component.$$);
			}
		} catch (e) {
			// reset dirty state to not end up in a deadlocked state and then rethrow
			dirty_components.length = 0;
			flushidx = 0;
			throw e;
		}
		set_current_component(null);
		dirty_components.length = 0;
		flushidx = 0;
		while (binding_callbacks.length) binding_callbacks.pop()();
		// then, once components are updated, call
		// afterUpdate functions. This may cause
		// subsequent updates...
		for (let i = 0; i < render_callbacks.length; i += 1) {
			const callback = render_callbacks[i];
			if (!seen_callbacks.has(callback)) {
				// ...so guard against infinite loops
				seen_callbacks.add(callback);
				callback();
			}
		}
		render_callbacks.length = 0;
	} while (dirty_components.length);
	while (flush_callbacks.length) {
		flush_callbacks.pop()();
	}
	update_scheduled = false;
	seen_callbacks.clear();
	set_current_component(saved_component);
}

/** @returns {void} */
function update($$) {
	if ($$.fragment !== null) {
		$$.update();
		run_all($$.before_update);
		const dirty = $$.dirty;
		$$.dirty = [-1];
		$$.fragment && $$.fragment.p($$.ctx, dirty);
		$$.after_update.forEach(add_render_callback);
	}
}

/**
 * Useful for example to execute remaining `afterUpdate` callbacks before executing `destroy`.
 * @param {Function[]} fns
 * @returns {void}
 */
function flush_render_callbacks(fns) {
	const filtered = [];
	const targets = [];
	render_callbacks.forEach((c) => (fns.indexOf(c) === -1 ? filtered.push(c) : targets.push(c)));
	targets.forEach((c) => c());
	render_callbacks = filtered;
}

const outroing = new Set();

/**
 * @type {Outro}
 */
let outros;

/**
 * @returns {void} */
function group_outros() {
	outros = {
		r: 0,
		c: [],
		p: outros // parent group
	};
}

/**
 * @returns {void} */
function check_outros() {
	if (!outros.r) {
		run_all(outros.c);
	}
	outros = outros.p;
}

/**
 * @param {import('./private.js').Fragment} block
 * @param {0 | 1} [local]
 * @returns {void}
 */
function transition_in(block, local) {
	if (block && block.i) {
		outroing.delete(block);
		block.i(local);
	}
}

/**
 * @param {import('./private.js').Fragment} block
 * @param {0 | 1} local
 * @param {0 | 1} [detach]
 * @param {() => void} [callback]
 * @returns {void}
 */
function transition_out(block, local, detach, callback) {
	if (block && block.o) {
		if (outroing.has(block)) return;
		outroing.add(block);
		outros.c.push(() => {
			outroing.delete(block);
			if (callback) {
				if (detach) block.d(1);
				callback();
			}
		});
		block.o(local);
	} else if (callback) {
		callback();
	}
}

/** @typedef {1} INTRO */
/** @typedef {0} OUTRO */
/** @typedef {{ direction: 'in' | 'out' | 'both' }} TransitionOptions */
/** @typedef {(node: Element, params: any, options: TransitionOptions) => import('../transition/public.js').TransitionConfig} TransitionFn */

/**
 * @typedef {Object} Outro
 * @property {number} r
 * @property {Function[]} c
 * @property {Object} p
 */

/**
 * @typedef {Object} PendingProgram
 * @property {number} start
 * @property {INTRO|OUTRO} b
 * @property {Outro} [group]
 */

/**
 * @typedef {Object} Program
 * @property {number} a
 * @property {INTRO|OUTRO} b
 * @property {1|-1} d
 * @property {number} duration
 * @property {number} start
 * @property {number} end
 * @property {Outro} [group]
 */

/** @returns {void} */
function create_component(block) {
	block && block.c();
}

/** @returns {void} */
function mount_component(component, target, anchor) {
	const { fragment, after_update } = component.$$;
	fragment && fragment.m(target, anchor);
	// onMount happens before the initial afterUpdate
	add_render_callback(() => {
		const new_on_destroy = component.$$.on_mount.map(run).filter(is_function);
		// if the component was destroyed immediately
		// it will update the `$$.on_destroy` reference to `null`.
		// the destructured on_destroy may still reference to the old array
		if (component.$$.on_destroy) {
			component.$$.on_destroy.push(...new_on_destroy);
		} else {
			// Edge case - component was destroyed immediately,
			// most likely as a result of a binding initialising
			run_all(new_on_destroy);
		}
		component.$$.on_mount = [];
	});
	after_update.forEach(add_render_callback);
}

/** @returns {void} */
function destroy_component(component, detaching) {
	const $$ = component.$$;
	if ($$.fragment !== null) {
		flush_render_callbacks($$.after_update);
		run_all($$.on_destroy);
		$$.fragment && $$.fragment.d(detaching);
		// TODO null out other refs, including component.$$ (but need to
		// preserve final state?)
		$$.on_destroy = $$.fragment = null;
		$$.ctx = [];
	}
}

/** @returns {void} */
function make_dirty(component, i) {
	if (component.$$.dirty[0] === -1) {
		dirty_components.push(component);
		schedule_update();
		component.$$.dirty.fill(0);
	}
	component.$$.dirty[(i / 31) | 0] |= 1 << i % 31;
}

// TODO: Document the other params
/**
 * @param {SvelteComponent} component
 * @param {import('./public.js').ComponentConstructorOptions} options
 *
 * @param {import('./utils.js')['not_equal']} not_equal Used to compare props and state values.
 * @param {(target: Element | ShadowRoot) => void} [append_styles] Function that appends styles to the DOM when the component is first initialised.
 * This will be the `add_css` function from the compiled component.
 *
 * @returns {void}
 */
function init(
	component,
	options,
	instance,
	create_fragment,
	not_equal,
	props,
	append_styles = null,
	dirty = [-1]
) {
	const parent_component = current_component;
	set_current_component(component);
	/** @type {import('./private.js').T$$} */
	const $$ = (component.$$ = {
		fragment: null,
		ctx: [],
		// state
		props,
		update: noop,
		not_equal,
		bound: blank_object(),
		// lifecycle
		on_mount: [],
		on_destroy: [],
		on_disconnect: [],
		before_update: [],
		after_update: [],
		context: new Map(options.context || (parent_component ? parent_component.$$.context : [])),
		// everything else
		callbacks: blank_object(),
		dirty,
		skip_bound: false,
		root: options.target || parent_component.$$.root
	});
	append_styles && append_styles($$.root);
	let ready = false;
	$$.ctx = instance
		? instance(component, options.props || {}, (i, ret, ...rest) => {
				const value = rest.length ? rest[0] : ret;
				if ($$.ctx && not_equal($$.ctx[i], ($$.ctx[i] = value))) {
					if (!$$.skip_bound && $$.bound[i]) $$.bound[i](value);
					if (ready) make_dirty(component, i);
				}
				return ret;
		  })
		: [];
	$$.update();
	ready = true;
	run_all($$.before_update);
	// `false` as a special case of no DOM component
	$$.fragment = create_fragment ? create_fragment($$.ctx) : false;
	if (options.target) {
		if (options.hydrate) {
			// TODO: what is the correct type here?
			// @ts-expect-error
			const nodes = children(options.target);
			$$.fragment && $$.fragment.l(nodes);
			nodes.forEach(detach);
		} else {
			// eslint-disable-next-line @typescript-eslint/no-non-null-assertion
			$$.fragment && $$.fragment.c();
		}
		if (options.intro) transition_in(component.$$.fragment);
		mount_component(component, options.target, options.anchor);
		flush();
	}
	set_current_component(parent_component);
}

/**
 * Base class for Svelte components. Used when dev=false.
 *
 * @template {Record<string, any>} [Props=any]
 * @template {Record<string, any>} [Events=any]
 */
class SvelteComponent {
	/**
	 * ### PRIVATE API
	 *
	 * Do not use, may change at any time
	 *
	 * @type {any}
	 */
	$$ = undefined;
	/**
	 * ### PRIVATE API
	 *
	 * Do not use, may change at any time
	 *
	 * @type {any}
	 */
	$$set = undefined;

	/** @returns {void} */
	$destroy() {
		destroy_component(this, 1);
		this.$destroy = noop;
	}

	/**
	 * @template {Extract<keyof Events, string>} K
	 * @param {K} type
	 * @param {((e: Events[K]) => void) | null | undefined} callback
	 * @returns {() => void}
	 */
	$on(type, callback) {
		if (!is_function(callback)) {
			return noop;
		}
		const callbacks = this.$$.callbacks[type] || (this.$$.callbacks[type] = []);
		callbacks.push(callback);
		return () => {
			const index = callbacks.indexOf(callback);
			if (index !== -1) callbacks.splice(index, 1);
		};
	}

	/**
	 * @param {Partial<Props>} props
	 * @returns {void}
	 */
	$set(props) {
		if (this.$$set && !is_empty(props)) {
			this.$$.skip_bound = true;
			this.$$set(props);
			this.$$.skip_bound = false;
		}
	}
}

/**
 * @typedef {Object} CustomElementPropDefinition
 * @property {string} [attribute]
 * @property {boolean} [reflect]
 * @property {'String'|'Boolean'|'Number'|'Array'|'Object'} [type]
 */

// generated during release, do not modify

const PUBLIC_VERSION = '4';

if (typeof window !== 'undefined')
	// @ts-ignore
	(window.__svelte || (window.__svelte = { v: new Set() })).v.add(PUBLIC_VERSION);

async function startEthereum(handleAccountsChanged) {
    const provider = await detectEthereumProvider();
    if(!provider) {
        console.log('Please install MetaMask!');
        setTimeout(startEthereum, 1000);
    } else {
        // If the provider returned by detectEthereumProvider is not the same as
        // window.ethereum, something is overwriting it, perhaps another wallet.
        if (provider !== window.ethereum) {
            console.error('Do you have multiple wallets installed?');
        }
        // Access the decentralized web!

        window.ethereum.on('chainChanged', window.location.reload);

        ethereum
            .request({ method: 'eth_accounts' })
            .then(handleAccountsChanged)
            .catch((err) => {
                // Some unexpected error.
                // For backwards compatibility reasons, if no accounts are available,
                // eth_accounts will return an empty array.
                console.error(err);
            });

        // Note that this event is emitted on page load.
        // If the array of accounts is non-empty, you're already
        // connected.
        ethereum.on('accountsChanged', handleAccountsChanged);
    }
}

const domain = [
    { name: "name", type: "string" },
    { name: "version", type: "string" },
    { name: "chainId", type: "uint256" },
];

const didEntangle = [
    { name: "did", type: "string" }
];

async function sign(signer, did) {
    const domainData = {
        name: "Ad4m DID Authentification",
        version: "2",
        //TODO: infer env from process.env and then use correct chainId; ropstein or eth
        chainId: 1,
    };

    var message = {
        did: did,
    };

    const data = {
        types: {
            EIP712Domain: domain,
            DIDEntangle: didEntangle,
        },
        domain: domainData,
        primaryType: "DIDEntangle",
        message: message
    };

    let req = {
        method: "eth_signTypedData_v3",
        params: [signer, JSON.stringify(data)],
        from: signer
    };
    console.log('sign: sending request ' + JSON.stringify(req));
    const result = await ethereum.request(req);

    console.log('sign: response received ' + result);

    const signature = result.substring(2);
    const r = "0x" + signature.substring(0, 64);
    const s = "0x" + signature.substring(64, 128);
    const v = parseInt(signature.substring(128, 130), 16);

    return { data, signature, r, s, v }
}

function connectWallet(handleAccountsChanged) {
    ethereum
        .request({ method: 'eth_requestAccounts' })
        .then(handleAccountsChanged)
        .catch((err) => {
            if (err.code === 4001) {
                // EIP-1193 userRejectedRequest error
                // If this happens, the user rejected the connection request.
                console.log('Please connect to MetaMask.');
            } else {
                console.error(err);
            }
        });
}

/******************************************************************************
Copyright (c) Microsoft Corporation.

Permission to use, copy, modify, and/or distribute this software for any
purpose with or without fee is hereby granted.

THE SOFTWARE IS PROVIDED "AS IS" AND THE AUTHOR DISCLAIMS ALL WARRANTIES WITH
REGARD TO THIS SOFTWARE INCLUDING ALL IMPLIED WARRANTIES OF MERCHANTABILITY
AND FITNESS. IN NO EVENT SHALL THE AUTHOR BE LIABLE FOR ANY SPECIAL, DIRECT,
INDIRECT, OR CONSEQUENTIAL DAMAGES OR ANY DAMAGES WHATSOEVER RESULTING FROM
LOSS OF USE, DATA OR PROFITS, WHETHER IN AN ACTION OF CONTRACT, NEGLIGENCE OR
OTHER TORTIOUS ACTION, ARISING OUT OF OR IN CONNECTION WITH THE USE OR
PERFORMANCE OF THIS SOFTWARE.
***************************************************************************** */
/* global Reflect, Promise, SuppressedError, Symbol */

var extendStatics = function(d, b) {
  extendStatics = Object.setPrototypeOf ||
      ({ __proto__: [] } instanceof Array && function (d, b) { d.__proto__ = b; }) ||
      function (d, b) { for (var p in b) if (Object.prototype.hasOwnProperty.call(b, p)) d[p] = b[p]; };
  return extendStatics(d, b);
};

function __extends(d, b) {
  if (typeof b !== "function" && b !== null)
      throw new TypeError("Class extends value " + String(b) + " is not a constructor or null");
  extendStatics(d, b);
  function __() { this.constructor = d; }
  d.prototype = b === null ? Object.create(b) : (__.prototype = b.prototype, new __());
}

var __assign$2 = function() {
  __assign$2 = Object.assign || function __assign(t) {
      for (var s, i = 1, n = arguments.length; i < n; i++) {
          s = arguments[i];
          for (var p in s) if (Object.prototype.hasOwnProperty.call(s, p)) t[p] = s[p];
      }
      return t;
  };
  return __assign$2.apply(this, arguments);
};

function __rest(s, e) {
  var t = {};
  for (var p in s) if (Object.prototype.hasOwnProperty.call(s, p) && e.indexOf(p) < 0)
      t[p] = s[p];
  if (s != null && typeof Object.getOwnPropertySymbols === "function")
      for (var i = 0, p = Object.getOwnPropertySymbols(s); i < p.length; i++) {
          if (e.indexOf(p[i]) < 0 && Object.prototype.propertyIsEnumerable.call(s, p[i]))
              t[p[i]] = s[p[i]];
      }
  return t;
}

function __awaiter$1(thisArg, _arguments, P, generator) {
  function adopt(value) { return value instanceof P ? value : new P(function (resolve) { resolve(value); }); }
  return new (P || (P = Promise))(function (resolve, reject) {
      function fulfilled(value) { try { step(generator.next(value)); } catch (e) { reject(e); } }
      function rejected(value) { try { step(generator["throw"](value)); } catch (e) { reject(e); } }
      function step(result) { result.done ? resolve(result.value) : adopt(result.value).then(fulfilled, rejected); }
      step((generator = generator.apply(thisArg, _arguments || [])).next());
  });
}

function __generator$1(thisArg, body) {
  var _ = { label: 0, sent: function() { if (t[0] & 1) throw t[1]; return t[1]; }, trys: [], ops: [] }, f, y, t, g;
  return g = { next: verb(0), "throw": verb(1), "return": verb(2) }, typeof Symbol === "function" && (g[Symbol.iterator] = function() { return this; }), g;
  function verb(n) { return function (v) { return step([n, v]); }; }
  function step(op) {
      if (f) throw new TypeError("Generator is already executing.");
      while (g && (g = 0, op[0] && (_ = 0)), _) try {
          if (f = 1, y && (t = op[0] & 2 ? y["return"] : op[0] ? y["throw"] || ((t = y["return"]) && t.call(y), 0) : y.next) && !(t = t.call(y, op[1])).done) return t;
          if (y = 0, t) op = [op[0] & 2, t.value];
          switch (op[0]) {
              case 0: case 1: t = op; break;
              case 4: _.label++; return { value: op[1], done: false };
              case 5: _.label++; y = op[1]; op = [0]; continue;
              case 7: op = _.ops.pop(); _.trys.pop(); continue;
              default:
                  if (!(t = _.trys, t = t.length > 0 && t[t.length - 1]) && (op[0] === 6 || op[0] === 2)) { _ = 0; continue; }
                  if (op[0] === 3 && (!t || (op[1] > t[0] && op[1] < t[3]))) { _.label = op[1]; break; }
                  if (op[0] === 6 && _.label < t[1]) { _.label = t[1]; t = op; break; }
                  if (t && _.label < t[2]) { _.label = t[2]; _.ops.push(op); break; }
                  if (t[2]) _.ops.pop();
                  _.trys.pop(); continue;
          }
          op = body.call(thisArg, _);
      } catch (e) { op = [6, e]; y = 0; } finally { f = t = 0; }
      if (op[0] & 5) throw op[1]; return { value: op[0] ? op[1] : void 0, done: true };
  }
}

function __spreadArray$1(to, from, pack) {
  if (pack || arguments.length === 2) for (var i = 0, l = from.length, ar; i < l; i++) {
      if (ar || !(i in from)) {
          if (!ar) ar = Array.prototype.slice.call(from, 0, i);
          ar[i] = from[i];
      }
  }
  return to.concat(ar || Array.prototype.slice.call(from));
}

typeof SuppressedError === "function" ? SuppressedError : function (error, suppressed, message) {
  var e = new Error(message);
  return e.name = "SuppressedError", e.error = error, e.suppressed = suppressed, e;
};

var genericMessage = "Invariant Violation";
var _a$2 = Object.setPrototypeOf, setPrototypeOf = _a$2 === void 0 ? function (obj, proto) {
    obj.__proto__ = proto;
    return obj;
} : _a$2;
var InvariantError = /** @class */ (function (_super) {
    __extends(InvariantError, _super);
    function InvariantError(message) {
        if (message === void 0) { message = genericMessage; }
        var _this = _super.call(this, typeof message === "number"
            ? genericMessage + ": " + message + " (see https://github.com/apollographql/invariant-packages)"
            : message) || this;
        _this.framesToPop = 1;
        _this.name = genericMessage;
        setPrototypeOf(_this, InvariantError.prototype);
        return _this;
    }
    return InvariantError;
}(Error));
function invariant$2(condition, message) {
    if (!condition) {
        throw new InvariantError(message);
    }
}
var verbosityLevels = ["debug", "log", "warn", "error", "silent"];
var verbosityLevel = verbosityLevels.indexOf("log");
function wrapConsoleMethod(name) {
    return function () {
        if (verbosityLevels.indexOf(name) >= verbosityLevel) {
            // Default to console.log if this host environment happens not to provide
            // all the console.* methods we need.
            var method = console[name] || console.log;
            return method.apply(console, arguments);
        }
    };
}
(function (invariant) {
    invariant.debug = wrapConsoleMethod("debug");
    invariant.log = wrapConsoleMethod("log");
    invariant.warn = wrapConsoleMethod("warn");
    invariant.error = wrapConsoleMethod("error");
})(invariant$2 || (invariant$2 = {}));

function maybe$2(thunk) {
    try {
        return thunk();
    }
    catch (_a) { }
}

var global$1 = (maybe$2(function () { return globalThis; }) ||
    maybe$2(function () { return window; }) ||
    maybe$2(function () { return self; }) ||
    maybe$2(function () { return global; }) || maybe$2(function () { return maybe$2.constructor("return this")(); }));

var __ = "__";
var GLOBAL_KEY = [__, __].join("DEV");
function getDEV() {
    try {
        return Boolean(__DEV__);
    }
    catch (_a) {
        Object.defineProperty(global$1, GLOBAL_KEY, {
            value: maybe$2(function () { return process.env.NODE_ENV; }) !== "production",
            enumerable: false,
            configurable: true,
            writable: true,
        });
        return global$1[GLOBAL_KEY];
    }
}
var DEV = getDEV();

function maybe$1(thunk) {
  try { return thunk() } catch (_) {}
}

var safeGlobal = (
  maybe$1(function() { return globalThis }) ||
  maybe$1(function() { return window }) ||
  maybe$1(function() { return self }) ||
  maybe$1(function() { return global }) ||
  // We don't expect the Function constructor ever to be invoked at runtime, as
  // long as at least one of globalThis, window, self, or global is defined, so
  // we are under no obligation to make it easy for static analysis tools to
  // detect syntactic usage of the Function constructor. If you think you can
  // improve your static analysis to detect this obfuscation, think again. This
  // is an arms race you cannot win, at least not in JavaScript.
  maybe$1(function() { return maybe$1.constructor("return this")() })
);

var needToRemove = false;

function install() {
  if (safeGlobal &&
      !maybe$1(function() { return process.env.NODE_ENV }) &&
      !maybe$1(function() { return process })) {
    Object.defineProperty(safeGlobal, "process", {
      value: {
        env: {
          // This default needs to be "production" instead of "development", to
          // avoid the problem https://github.com/graphql/graphql-js/pull/2894
          // will eventually solve, once merged and released.
          NODE_ENV: "production",
        },
      },
      // Let anyone else change global.process as they see fit, but hide it from
      // Object.keys(global) enumeration.
      configurable: true,
      enumerable: false,
      writable: true,
    });
    needToRemove = true;
  }
}

// Call install() at least once, when this module is imported.
install();

function remove() {
  if (needToRemove) {
    delete safeGlobal.process;
    needToRemove = false;
  }
}

// In ES2015 (or a polyfilled) environment, this will be Symbol.iterator
// istanbul ignore next (See: 'https://github.com/graphql/graphql-js/issues/2317')

var SYMBOL_TO_STRING_TAG$1 = typeof Symbol === 'function' && Symbol.toStringTag != null ? Symbol.toStringTag : '@@toStringTag';

/**
 * The set of allowed kind values for AST nodes.
 */
var Kind$1 = Object.freeze({
  // Name
  NAME: 'Name',
  // Document
  DOCUMENT: 'Document',
  OPERATION_DEFINITION: 'OperationDefinition',
  VARIABLE_DEFINITION: 'VariableDefinition',
  SELECTION_SET: 'SelectionSet',
  FIELD: 'Field',
  ARGUMENT: 'Argument',
  // Fragments
  FRAGMENT_SPREAD: 'FragmentSpread',
  INLINE_FRAGMENT: 'InlineFragment',
  FRAGMENT_DEFINITION: 'FragmentDefinition',
  // Values
  VARIABLE: 'Variable',
  INT: 'IntValue',
  FLOAT: 'FloatValue',
  STRING: 'StringValue',
  BOOLEAN: 'BooleanValue',
  NULL: 'NullValue',
  ENUM: 'EnumValue',
  LIST: 'ListValue',
  OBJECT: 'ObjectValue',
  OBJECT_FIELD: 'ObjectField',
  // Directives
  DIRECTIVE: 'Directive',
  // Types
  NAMED_TYPE: 'NamedType',
  LIST_TYPE: 'ListType',
  NON_NULL_TYPE: 'NonNullType',
  // Type System Definitions
  SCHEMA_DEFINITION: 'SchemaDefinition',
  OPERATION_TYPE_DEFINITION: 'OperationTypeDefinition',
  // Type Definitions
  SCALAR_TYPE_DEFINITION: 'ScalarTypeDefinition',
  OBJECT_TYPE_DEFINITION: 'ObjectTypeDefinition',
  FIELD_DEFINITION: 'FieldDefinition',
  INPUT_VALUE_DEFINITION: 'InputValueDefinition',
  INTERFACE_TYPE_DEFINITION: 'InterfaceTypeDefinition',
  UNION_TYPE_DEFINITION: 'UnionTypeDefinition',
  ENUM_TYPE_DEFINITION: 'EnumTypeDefinition',
  ENUM_VALUE_DEFINITION: 'EnumValueDefinition',
  INPUT_OBJECT_TYPE_DEFINITION: 'InputObjectTypeDefinition',
  // Directive Definitions
  DIRECTIVE_DEFINITION: 'DirectiveDefinition',
  // Type System Extensions
  SCHEMA_EXTENSION: 'SchemaExtension',
  // Type Extensions
  SCALAR_TYPE_EXTENSION: 'ScalarTypeExtension',
  OBJECT_TYPE_EXTENSION: 'ObjectTypeExtension',
  INTERFACE_TYPE_EXTENSION: 'InterfaceTypeExtension',
  UNION_TYPE_EXTENSION: 'UnionTypeExtension',
  ENUM_TYPE_EXTENSION: 'EnumTypeExtension',
  INPUT_OBJECT_TYPE_EXTENSION: 'InputObjectTypeExtension'
});
/**
 * The enum type representing the possible kind values of AST nodes.
 */

function invariant$1(condition, message) {
  var booleanCondition = Boolean(condition); // istanbul ignore else (See transformation done in './resources/inlineInvariant.js')

  if (!booleanCondition) {
    throw new Error(message != null ? message : 'Unexpected invariant triggered.');
  }
}

// istanbul ignore next (See: 'https://github.com/graphql/graphql-js/issues/2317')
var nodejsCustomInspectSymbol$2 = typeof Symbol === 'function' && typeof Symbol.for === 'function' ? Symbol.for('nodejs.util.inspect.custom') : undefined;
var nodejsCustomInspectSymbol$3 = nodejsCustomInspectSymbol$2;

/**
 * The `defineInspect()` function defines `inspect()` prototype method as alias of `toJSON`
 */

function defineInspect$1(classObject) {
  var fn = classObject.prototype.toJSON;
  typeof fn === 'function' || invariant$1(0);
  classObject.prototype.inspect = fn; // istanbul ignore else (See: 'https://github.com/graphql/graphql-js/issues/2317')

  if (nodejsCustomInspectSymbol$3) {
    classObject.prototype[nodejsCustomInspectSymbol$3] = fn;
  }
}

/**
 * Contains a range of UTF-8 character offsets and token references that
 * identify the region of the source from which the AST derived.
 */
var Location$1 = /*#__PURE__*/function () {
  /**
   * The character offset at which this Node begins.
   */

  /**
   * The character offset at which this Node ends.
   */

  /**
   * The Token at which this Node begins.
   */

  /**
   * The Token at which this Node ends.
   */

  /**
   * The Source document the AST represents.
   */
  function Location(startToken, endToken, source) {
    this.start = startToken.start;
    this.end = endToken.end;
    this.startToken = startToken;
    this.endToken = endToken;
    this.source = source;
  }

  var _proto = Location.prototype;

  _proto.toJSON = function toJSON() {
    return {
      start: this.start,
      end: this.end
    };
  };

  return Location;
}(); // Print a simplified form when appearing in `inspect` and `util.inspect`.

defineInspect$1(Location$1);
/**
 * Represents a range of characters represented by a lexical token
 * within a Source.
 */

var Token$1 = /*#__PURE__*/function () {
  /**
   * The kind of Token.
   */

  /**
   * The character offset at which this Node begins.
   */

  /**
   * The character offset at which this Node ends.
   */

  /**
   * The 1-indexed line number on which this Token appears.
   */

  /**
   * The 1-indexed column number at which this Token begins.
   */

  /**
   * For non-punctuation tokens, represents the interpreted value of the token.
   */

  /**
   * Tokens exist as nodes in a double-linked-list amongst all tokens
   * including ignored tokens. <SOF> is always the first node and <EOF>
   * the last.
   */
  function Token(kind, start, end, line, column, prev, value) {
    this.kind = kind;
    this.start = start;
    this.end = end;
    this.line = line;
    this.column = column;
    this.value = value;
    this.prev = prev;
    this.next = null;
  }

  var _proto2 = Token.prototype;

  _proto2.toJSON = function toJSON() {
    return {
      kind: this.kind,
      value: this.value,
      line: this.line,
      column: this.column
    };
  };

  return Token;
}(); // Print a simplified form when appearing in `inspect` and `util.inspect`.

defineInspect$1(Token$1);
/**
 * @internal
 */

function isNode(maybeNode) {
  return maybeNode != null && typeof maybeNode.kind === 'string';
}
/**
 * The list of all possible AST node types.
 */

function _typeof$3(obj) { "@babel/helpers - typeof"; if (typeof Symbol === "function" && typeof Symbol.iterator === "symbol") { _typeof$3 = function _typeof(obj) { return typeof obj; }; } else { _typeof$3 = function _typeof(obj) { return obj && typeof Symbol === "function" && obj.constructor === Symbol && obj !== Symbol.prototype ? "symbol" : typeof obj; }; } return _typeof$3(obj); }
var MAX_ARRAY_LENGTH$1 = 10;
var MAX_RECURSIVE_DEPTH$1 = 2;
/**
 * Used to print values in error messages.
 */

function inspect$1(value) {
  return formatValue$1(value, []);
}

function formatValue$1(value, seenValues) {
  switch (_typeof$3(value)) {
    case 'string':
      return JSON.stringify(value);

    case 'function':
      return value.name ? "[function ".concat(value.name, "]") : '[function]';

    case 'object':
      if (value === null) {
        return 'null';
      }

      return formatObjectValue$1(value, seenValues);

    default:
      return String(value);
  }
}

function formatObjectValue$1(value, previouslySeenValues) {
  if (previouslySeenValues.indexOf(value) !== -1) {
    return '[Circular]';
  }

  var seenValues = [].concat(previouslySeenValues, [value]);
  var customInspectFn = getCustomFn$1(value);

  if (customInspectFn !== undefined) {
    var customValue = customInspectFn.call(value); // check for infinite recursion

    if (customValue !== value) {
      return typeof customValue === 'string' ? customValue : formatValue$1(customValue, seenValues);
    }
  } else if (Array.isArray(value)) {
    return formatArray$1(value, seenValues);
  }

  return formatObject$1(value, seenValues);
}

function formatObject$1(object, seenValues) {
  var keys = Object.keys(object);

  if (keys.length === 0) {
    return '{}';
  }

  if (seenValues.length > MAX_RECURSIVE_DEPTH$1) {
    return '[' + getObjectTag$1(object) + ']';
  }

  var properties = keys.map(function (key) {
    var value = formatValue$1(object[key], seenValues);
    return key + ': ' + value;
  });
  return '{ ' + properties.join(', ') + ' }';
}

function formatArray$1(array, seenValues) {
  if (array.length === 0) {
    return '[]';
  }

  if (seenValues.length > MAX_RECURSIVE_DEPTH$1) {
    return '[Array]';
  }

  var len = Math.min(MAX_ARRAY_LENGTH$1, array.length);
  var remaining = array.length - len;
  var items = [];

  for (var i = 0; i < len; ++i) {
    items.push(formatValue$1(array[i], seenValues));
  }

  if (remaining === 1) {
    items.push('... 1 more item');
  } else if (remaining > 1) {
    items.push("... ".concat(remaining, " more items"));
  }

  return '[' + items.join(', ') + ']';
}

function getCustomFn$1(object) {
  var customInspectFn = object[String(nodejsCustomInspectSymbol$3)];

  if (typeof customInspectFn === 'function') {
    return customInspectFn;
  }

  if (typeof object.inspect === 'function') {
    return object.inspect;
  }
}

function getObjectTag$1(object) {
  var tag = Object.prototype.toString.call(object).replace(/^\[object /, '').replace(/]$/, '');

  if (tag === 'Object' && typeof object.constructor === 'function') {
    var name = object.constructor.name;

    if (typeof name === 'string' && name !== '') {
      return name;
    }
  }

  return tag;
}

function devAssert$1(condition, message) {
  var booleanCondition = Boolean(condition); // istanbul ignore else (See transformation done in './resources/inlineInvariant.js')

  if (!booleanCondition) {
    throw new Error(message);
  }
}

function _defineProperties$3(target, props) { for (var i = 0; i < props.length; i++) { var descriptor = props[i]; descriptor.enumerable = descriptor.enumerable || false; descriptor.configurable = true; if ("value" in descriptor) descriptor.writable = true; Object.defineProperty(target, descriptor.key, descriptor); } }

function _createClass$3(Constructor, protoProps, staticProps) { if (protoProps) _defineProperties$3(Constructor.prototype, protoProps); if (staticProps) _defineProperties$3(Constructor, staticProps); return Constructor; }

/**
 * A representation of source input to GraphQL. The `name` and `locationOffset` parameters are
 * optional, but they are useful for clients who store GraphQL documents in source files.
 * For example, if the GraphQL input starts at line 40 in a file named `Foo.graphql`, it might
 * be useful for `name` to be `"Foo.graphql"` and location to be `{ line: 40, column: 1 }`.
 * The `line` and `column` properties in `locationOffset` are 1-indexed.
 */
var Source$1 = /*#__PURE__*/function () {
  function Source(body) {
    var name = arguments.length > 1 && arguments[1] !== undefined ? arguments[1] : 'GraphQL request';
    var locationOffset = arguments.length > 2 && arguments[2] !== undefined ? arguments[2] : {
      line: 1,
      column: 1
    };
    typeof body === 'string' || devAssert$1(0, "Body must be a string. Received: ".concat(inspect$1(body), "."));
    this.body = body;
    this.name = name;
    this.locationOffset = locationOffset;
    this.locationOffset.line > 0 || devAssert$1(0, 'line in locationOffset is 1-indexed and must be positive.');
    this.locationOffset.column > 0 || devAssert$1(0, 'column in locationOffset is 1-indexed and must be positive.');
  } // $FlowFixMe[unsupported-syntax] Flow doesn't support computed properties yet


  _createClass$3(Source, [{
    key: SYMBOL_TO_STRING_TAG$1,
    get: function get() {
      return 'Source';
    }
  }]);

  return Source;
}();

/**
 * Produces the value of a block string from its parsed raw value, similar to
 * CoffeeScript's block string, Python's docstring trim or Ruby's strip_heredoc.
 *
 * This implements the GraphQL spec's BlockStringValue() static algorithm.
 *
 * @internal
 */
/**
 * Print a block string in the indented block form by adding a leading and
 * trailing blank line. However, if a block string starts with whitespace and is
 * a single-line, adding a leading blank line would strip that whitespace.
 *
 * @internal
 */

function printBlockString(value) {
  var indentation = arguments.length > 1 && arguments[1] !== undefined ? arguments[1] : '';
  var preferMultipleLines = arguments.length > 2 && arguments[2] !== undefined ? arguments[2] : false;
  var isSingleLine = value.indexOf('\n') === -1;
  var hasLeadingSpace = value[0] === ' ' || value[0] === '\t';
  var hasTrailingQuote = value[value.length - 1] === '"';
  var hasTrailingSlash = value[value.length - 1] === '\\';
  var printAsMultipleLines = !isSingleLine || hasTrailingQuote || hasTrailingSlash || preferMultipleLines;
  var result = ''; // Format a multi-line block quote to account for leading space.

  if (printAsMultipleLines && !(isSingleLine && hasLeadingSpace)) {
    result += '\n' + indentation;
  }

  result += indentation ? value.replace(/\n/g, '\n' + indentation) : value;

  if (printAsMultipleLines) {
    result += '\n';
  }

  return '"""' + result.replace(/"""/g, '\\"""') + '"""';
}

/**
 * A visitor is provided to visit, it contains the collection of
 * relevant functions to be called during the visitor's traversal.
 */

var QueryDocumentKeys = {
  Name: [],
  Document: ['definitions'],
  OperationDefinition: ['name', 'variableDefinitions', 'directives', 'selectionSet'],
  VariableDefinition: ['variable', 'type', 'defaultValue', 'directives'],
  Variable: ['name'],
  SelectionSet: ['selections'],
  Field: ['alias', 'name', 'arguments', 'directives', 'selectionSet'],
  Argument: ['name', 'value'],
  FragmentSpread: ['name', 'directives'],
  InlineFragment: ['typeCondition', 'directives', 'selectionSet'],
  FragmentDefinition: ['name', // Note: fragment variable definitions are experimental and may be changed
  // or removed in the future.
  'variableDefinitions', 'typeCondition', 'directives', 'selectionSet'],
  IntValue: [],
  FloatValue: [],
  StringValue: [],
  BooleanValue: [],
  NullValue: [],
  EnumValue: [],
  ListValue: ['values'],
  ObjectValue: ['fields'],
  ObjectField: ['name', 'value'],
  Directive: ['name', 'arguments'],
  NamedType: ['name'],
  ListType: ['type'],
  NonNullType: ['type'],
  SchemaDefinition: ['description', 'directives', 'operationTypes'],
  OperationTypeDefinition: ['type'],
  ScalarTypeDefinition: ['description', 'name', 'directives'],
  ObjectTypeDefinition: ['description', 'name', 'interfaces', 'directives', 'fields'],
  FieldDefinition: ['description', 'name', 'arguments', 'type', 'directives'],
  InputValueDefinition: ['description', 'name', 'type', 'defaultValue', 'directives'],
  InterfaceTypeDefinition: ['description', 'name', 'interfaces', 'directives', 'fields'],
  UnionTypeDefinition: ['description', 'name', 'directives', 'types'],
  EnumTypeDefinition: ['description', 'name', 'directives', 'values'],
  EnumValueDefinition: ['description', 'name', 'directives'],
  InputObjectTypeDefinition: ['description', 'name', 'directives', 'fields'],
  DirectiveDefinition: ['description', 'name', 'arguments', 'locations'],
  SchemaExtension: ['directives', 'operationTypes'],
  ScalarTypeExtension: ['name', 'directives'],
  ObjectTypeExtension: ['name', 'interfaces', 'directives', 'fields'],
  InterfaceTypeExtension: ['name', 'interfaces', 'directives', 'fields'],
  UnionTypeExtension: ['name', 'directives', 'types'],
  EnumTypeExtension: ['name', 'directives', 'values'],
  InputObjectTypeExtension: ['name', 'directives', 'fields']
};
var BREAK = Object.freeze({});
/**
 * visit() will walk through an AST using a depth-first traversal, calling
 * the visitor's enter function at each node in the traversal, and calling the
 * leave function after visiting that node and all of its child nodes.
 *
 * By returning different values from the enter and leave functions, the
 * behavior of the visitor can be altered, including skipping over a sub-tree of
 * the AST (by returning false), editing the AST by returning a value or null
 * to remove the value, or to stop the whole traversal by returning BREAK.
 *
 * When using visit() to edit an AST, the original AST will not be modified, and
 * a new version of the AST with the changes applied will be returned from the
 * visit function.
 *
 *     const editedAST = visit(ast, {
 *       enter(node, key, parent, path, ancestors) {
 *         // @return
 *         //   undefined: no action
 *         //   false: skip visiting this node
 *         //   visitor.BREAK: stop visiting altogether
 *         //   null: delete this node
 *         //   any value: replace this node with the returned value
 *       },
 *       leave(node, key, parent, path, ancestors) {
 *         // @return
 *         //   undefined: no action
 *         //   false: no action
 *         //   visitor.BREAK: stop visiting altogether
 *         //   null: delete this node
 *         //   any value: replace this node with the returned value
 *       }
 *     });
 *
 * Alternatively to providing enter() and leave() functions, a visitor can
 * instead provide functions named the same as the kinds of AST nodes, or
 * enter/leave visitors at a named key, leading to four permutations of the
 * visitor API:
 *
 * 1) Named visitors triggered when entering a node of a specific kind.
 *
 *     visit(ast, {
 *       Kind(node) {
 *         // enter the "Kind" node
 *       }
 *     })
 *
 * 2) Named visitors that trigger upon entering and leaving a node of
 *    a specific kind.
 *
 *     visit(ast, {
 *       Kind: {
 *         enter(node) {
 *           // enter the "Kind" node
 *         }
 *         leave(node) {
 *           // leave the "Kind" node
 *         }
 *       }
 *     })
 *
 * 3) Generic visitors that trigger upon entering and leaving any node.
 *
 *     visit(ast, {
 *       enter(node) {
 *         // enter any node
 *       },
 *       leave(node) {
 *         // leave any node
 *       }
 *     })
 *
 * 4) Parallel visitors for entering and leaving nodes of a specific kind.
 *
 *     visit(ast, {
 *       enter: {
 *         Kind(node) {
 *           // enter the "Kind" node
 *         }
 *       },
 *       leave: {
 *         Kind(node) {
 *           // leave the "Kind" node
 *         }
 *       }
 *     })
 */

function visit(root, visitor) {
  var visitorKeys = arguments.length > 2 && arguments[2] !== undefined ? arguments[2] : QueryDocumentKeys;

  /* eslint-disable no-undef-init */
  var stack = undefined;
  var inArray = Array.isArray(root);
  var keys = [root];
  var index = -1;
  var edits = [];
  var node = undefined;
  var key = undefined;
  var parent = undefined;
  var path = [];
  var ancestors = [];
  var newRoot = root;
  /* eslint-enable no-undef-init */

  do {
    index++;
    var isLeaving = index === keys.length;
    var isEdited = isLeaving && edits.length !== 0;

    if (isLeaving) {
      key = ancestors.length === 0 ? undefined : path[path.length - 1];
      node = parent;
      parent = ancestors.pop();

      if (isEdited) {
        if (inArray) {
          node = node.slice();
        } else {
          var clone = {};

          for (var _i2 = 0, _Object$keys2 = Object.keys(node); _i2 < _Object$keys2.length; _i2++) {
            var k = _Object$keys2[_i2];
            clone[k] = node[k];
          }

          node = clone;
        }

        var editOffset = 0;

        for (var ii = 0; ii < edits.length; ii++) {
          var editKey = edits[ii][0];
          var editValue = edits[ii][1];

          if (inArray) {
            editKey -= editOffset;
          }

          if (inArray && editValue === null) {
            node.splice(editKey, 1);
            editOffset++;
          } else {
            node[editKey] = editValue;
          }
        }
      }

      index = stack.index;
      keys = stack.keys;
      edits = stack.edits;
      inArray = stack.inArray;
      stack = stack.prev;
    } else {
      key = parent ? inArray ? index : keys[index] : undefined;
      node = parent ? parent[key] : newRoot;

      if (node === null || node === undefined) {
        continue;
      }

      if (parent) {
        path.push(key);
      }
    }

    var result = void 0;

    if (!Array.isArray(node)) {
      if (!isNode(node)) {
        throw new Error("Invalid AST Node: ".concat(inspect$1(node), "."));
      }

      var visitFn = getVisitFn(visitor, node.kind, isLeaving);

      if (visitFn) {
        result = visitFn.call(visitor, node, key, parent, path, ancestors);

        if (result === BREAK) {
          break;
        }

        if (result === false) {
          if (!isLeaving) {
            path.pop();
            continue;
          }
        } else if (result !== undefined) {
          edits.push([key, result]);

          if (!isLeaving) {
            if (isNode(result)) {
              node = result;
            } else {
              path.pop();
              continue;
            }
          }
        }
      }
    }

    if (result === undefined && isEdited) {
      edits.push([key, node]);
    }

    if (isLeaving) {
      path.pop();
    } else {
      var _visitorKeys$node$kin;

      stack = {
        inArray: inArray,
        index: index,
        keys: keys,
        edits: edits,
        prev: stack
      };
      inArray = Array.isArray(node);
      keys = inArray ? node : (_visitorKeys$node$kin = visitorKeys[node.kind]) !== null && _visitorKeys$node$kin !== void 0 ? _visitorKeys$node$kin : [];
      index = -1;
      edits = [];

      if (parent) {
        ancestors.push(parent);
      }

      parent = node;
    }
  } while (stack !== undefined);

  if (edits.length !== 0) {
    newRoot = edits[edits.length - 1][1];
  }

  return newRoot;
}
/**
 * Given a visitor instance, if it is leaving or not, and a node kind, return
 * the function the visitor runtime should call.
 */

function getVisitFn(visitor, kind, isLeaving) {
  var kindVisitor = visitor[kind];

  if (kindVisitor) {
    if (!isLeaving && typeof kindVisitor === 'function') {
      // { Kind() {} }
      return kindVisitor;
    }

    var kindSpecificVisitor = isLeaving ? kindVisitor.leave : kindVisitor.enter;

    if (typeof kindSpecificVisitor === 'function') {
      // { Kind: { enter() {}, leave() {} } }
      return kindSpecificVisitor;
    }
  } else {
    var specificVisitor = isLeaving ? visitor.leave : visitor.enter;

    if (specificVisitor) {
      if (typeof specificVisitor === 'function') {
        // { enter() {}, leave() {} }
        return specificVisitor;
      }

      var specificKindVisitor = specificVisitor[kind];

      if (typeof specificKindVisitor === 'function') {
        // { enter: { Kind() {} }, leave: { Kind() {} } }
        return specificKindVisitor;
      }
    }
  }
}

/**
 * Converts an AST into a string, using one set of reasonable
 * formatting rules.
 */

function print(ast) {
  return visit(ast, {
    leave: printDocASTReducer
  });
}
var MAX_LINE_LENGTH = 80; // TODO: provide better type coverage in future

var printDocASTReducer = {
  Name: function Name(node) {
    return node.value;
  },
  Variable: function Variable(node) {
    return '$' + node.name;
  },
  // Document
  Document: function Document(node) {
    return join(node.definitions, '\n\n') + '\n';
  },
  OperationDefinition: function OperationDefinition(node) {
    var op = node.operation;
    var name = node.name;
    var varDefs = wrap$1('(', join(node.variableDefinitions, ', '), ')');
    var directives = join(node.directives, ' ');
    var selectionSet = node.selectionSet; // Anonymous queries with no directives or variable definitions can use
    // the query short form.

    return !name && !directives && !varDefs && op === 'query' ? selectionSet : join([op, join([name, varDefs]), directives, selectionSet], ' ');
  },
  VariableDefinition: function VariableDefinition(_ref) {
    var variable = _ref.variable,
        type = _ref.type,
        defaultValue = _ref.defaultValue,
        directives = _ref.directives;
    return variable + ': ' + type + wrap$1(' = ', defaultValue) + wrap$1(' ', join(directives, ' '));
  },
  SelectionSet: function SelectionSet(_ref2) {
    var selections = _ref2.selections;
    return block(selections);
  },
  Field: function Field(_ref3) {
    var alias = _ref3.alias,
        name = _ref3.name,
        args = _ref3.arguments,
        directives = _ref3.directives,
        selectionSet = _ref3.selectionSet;
    var prefix = wrap$1('', alias, ': ') + name;
    var argsLine = prefix + wrap$1('(', join(args, ', '), ')');

    if (argsLine.length > MAX_LINE_LENGTH) {
      argsLine = prefix + wrap$1('(\n', indent(join(args, '\n')), '\n)');
    }

    return join([argsLine, join(directives, ' '), selectionSet], ' ');
  },
  Argument: function Argument(_ref4) {
    var name = _ref4.name,
        value = _ref4.value;
    return name + ': ' + value;
  },
  // Fragments
  FragmentSpread: function FragmentSpread(_ref5) {
    var name = _ref5.name,
        directives = _ref5.directives;
    return '...' + name + wrap$1(' ', join(directives, ' '));
  },
  InlineFragment: function InlineFragment(_ref6) {
    var typeCondition = _ref6.typeCondition,
        directives = _ref6.directives,
        selectionSet = _ref6.selectionSet;
    return join(['...', wrap$1('on ', typeCondition), join(directives, ' '), selectionSet], ' ');
  },
  FragmentDefinition: function FragmentDefinition(_ref7) {
    var name = _ref7.name,
        typeCondition = _ref7.typeCondition,
        variableDefinitions = _ref7.variableDefinitions,
        directives = _ref7.directives,
        selectionSet = _ref7.selectionSet;
    return (// Note: fragment variable definitions are experimental and may be changed
      // or removed in the future.
      "fragment ".concat(name).concat(wrap$1('(', join(variableDefinitions, ', '), ')'), " ") + "on ".concat(typeCondition, " ").concat(wrap$1('', join(directives, ' '), ' ')) + selectionSet
    );
  },
  // Value
  IntValue: function IntValue(_ref8) {
    var value = _ref8.value;
    return value;
  },
  FloatValue: function FloatValue(_ref9) {
    var value = _ref9.value;
    return value;
  },
  StringValue: function StringValue(_ref10, key) {
    var value = _ref10.value,
        isBlockString = _ref10.block;
    return isBlockString ? printBlockString(value, key === 'description' ? '' : '  ') : JSON.stringify(value);
  },
  BooleanValue: function BooleanValue(_ref11) {
    var value = _ref11.value;
    return value ? 'true' : 'false';
  },
  NullValue: function NullValue() {
    return 'null';
  },
  EnumValue: function EnumValue(_ref12) {
    var value = _ref12.value;
    return value;
  },
  ListValue: function ListValue(_ref13) {
    var values = _ref13.values;
    return '[' + join(values, ', ') + ']';
  },
  ObjectValue: function ObjectValue(_ref14) {
    var fields = _ref14.fields;
    return '{' + join(fields, ', ') + '}';
  },
  ObjectField: function ObjectField(_ref15) {
    var name = _ref15.name,
        value = _ref15.value;
    return name + ': ' + value;
  },
  // Directive
  Directive: function Directive(_ref16) {
    var name = _ref16.name,
        args = _ref16.arguments;
    return '@' + name + wrap$1('(', join(args, ', '), ')');
  },
  // Type
  NamedType: function NamedType(_ref17) {
    var name = _ref17.name;
    return name;
  },
  ListType: function ListType(_ref18) {
    var type = _ref18.type;
    return '[' + type + ']';
  },
  NonNullType: function NonNullType(_ref19) {
    var type = _ref19.type;
    return type + '!';
  },
  // Type System Definitions
  SchemaDefinition: addDescription(function (_ref20) {
    var directives = _ref20.directives,
        operationTypes = _ref20.operationTypes;
    return join(['schema', join(directives, ' '), block(operationTypes)], ' ');
  }),
  OperationTypeDefinition: function OperationTypeDefinition(_ref21) {
    var operation = _ref21.operation,
        type = _ref21.type;
    return operation + ': ' + type;
  },
  ScalarTypeDefinition: addDescription(function (_ref22) {
    var name = _ref22.name,
        directives = _ref22.directives;
    return join(['scalar', name, join(directives, ' ')], ' ');
  }),
  ObjectTypeDefinition: addDescription(function (_ref23) {
    var name = _ref23.name,
        interfaces = _ref23.interfaces,
        directives = _ref23.directives,
        fields = _ref23.fields;
    return join(['type', name, wrap$1('implements ', join(interfaces, ' & ')), join(directives, ' '), block(fields)], ' ');
  }),
  FieldDefinition: addDescription(function (_ref24) {
    var name = _ref24.name,
        args = _ref24.arguments,
        type = _ref24.type,
        directives = _ref24.directives;
    return name + (hasMultilineItems(args) ? wrap$1('(\n', indent(join(args, '\n')), '\n)') : wrap$1('(', join(args, ', '), ')')) + ': ' + type + wrap$1(' ', join(directives, ' '));
  }),
  InputValueDefinition: addDescription(function (_ref25) {
    var name = _ref25.name,
        type = _ref25.type,
        defaultValue = _ref25.defaultValue,
        directives = _ref25.directives;
    return join([name + ': ' + type, wrap$1('= ', defaultValue), join(directives, ' ')], ' ');
  }),
  InterfaceTypeDefinition: addDescription(function (_ref26) {
    var name = _ref26.name,
        interfaces = _ref26.interfaces,
        directives = _ref26.directives,
        fields = _ref26.fields;
    return join(['interface', name, wrap$1('implements ', join(interfaces, ' & ')), join(directives, ' '), block(fields)], ' ');
  }),
  UnionTypeDefinition: addDescription(function (_ref27) {
    var name = _ref27.name,
        directives = _ref27.directives,
        types = _ref27.types;
    return join(['union', name, join(directives, ' '), types && types.length !== 0 ? '= ' + join(types, ' | ') : ''], ' ');
  }),
  EnumTypeDefinition: addDescription(function (_ref28) {
    var name = _ref28.name,
        directives = _ref28.directives,
        values = _ref28.values;
    return join(['enum', name, join(directives, ' '), block(values)], ' ');
  }),
  EnumValueDefinition: addDescription(function (_ref29) {
    var name = _ref29.name,
        directives = _ref29.directives;
    return join([name, join(directives, ' ')], ' ');
  }),
  InputObjectTypeDefinition: addDescription(function (_ref30) {
    var name = _ref30.name,
        directives = _ref30.directives,
        fields = _ref30.fields;
    return join(['input', name, join(directives, ' '), block(fields)], ' ');
  }),
  DirectiveDefinition: addDescription(function (_ref31) {
    var name = _ref31.name,
        args = _ref31.arguments,
        repeatable = _ref31.repeatable,
        locations = _ref31.locations;
    return 'directive @' + name + (hasMultilineItems(args) ? wrap$1('(\n', indent(join(args, '\n')), '\n)') : wrap$1('(', join(args, ', '), ')')) + (repeatable ? ' repeatable' : '') + ' on ' + join(locations, ' | ');
  }),
  SchemaExtension: function SchemaExtension(_ref32) {
    var directives = _ref32.directives,
        operationTypes = _ref32.operationTypes;
    return join(['extend schema', join(directives, ' '), block(operationTypes)], ' ');
  },
  ScalarTypeExtension: function ScalarTypeExtension(_ref33) {
    var name = _ref33.name,
        directives = _ref33.directives;
    return join(['extend scalar', name, join(directives, ' ')], ' ');
  },
  ObjectTypeExtension: function ObjectTypeExtension(_ref34) {
    var name = _ref34.name,
        interfaces = _ref34.interfaces,
        directives = _ref34.directives,
        fields = _ref34.fields;
    return join(['extend type', name, wrap$1('implements ', join(interfaces, ' & ')), join(directives, ' '), block(fields)], ' ');
  },
  InterfaceTypeExtension: function InterfaceTypeExtension(_ref35) {
    var name = _ref35.name,
        interfaces = _ref35.interfaces,
        directives = _ref35.directives,
        fields = _ref35.fields;
    return join(['extend interface', name, wrap$1('implements ', join(interfaces, ' & ')), join(directives, ' '), block(fields)], ' ');
  },
  UnionTypeExtension: function UnionTypeExtension(_ref36) {
    var name = _ref36.name,
        directives = _ref36.directives,
        types = _ref36.types;
    return join(['extend union', name, join(directives, ' '), types && types.length !== 0 ? '= ' + join(types, ' | ') : ''], ' ');
  },
  EnumTypeExtension: function EnumTypeExtension(_ref37) {
    var name = _ref37.name,
        directives = _ref37.directives,
        values = _ref37.values;
    return join(['extend enum', name, join(directives, ' '), block(values)], ' ');
  },
  InputObjectTypeExtension: function InputObjectTypeExtension(_ref38) {
    var name = _ref38.name,
        directives = _ref38.directives,
        fields = _ref38.fields;
    return join(['extend input', name, join(directives, ' '), block(fields)], ' ');
  }
};

function addDescription(cb) {
  return function (node) {
    return join([node.description, cb(node)], '\n');
  };
}
/**
 * Given maybeArray, print an empty string if it is null or empty, otherwise
 * print all items together separated by separator if provided
 */


function join(maybeArray) {
  var _maybeArray$filter$jo;

  var separator = arguments.length > 1 && arguments[1] !== undefined ? arguments[1] : '';
  return (_maybeArray$filter$jo = maybeArray === null || maybeArray === void 0 ? void 0 : maybeArray.filter(function (x) {
    return x;
  }).join(separator)) !== null && _maybeArray$filter$jo !== void 0 ? _maybeArray$filter$jo : '';
}
/**
 * Given array, print each item on its own line, wrapped in an
 * indented "{ }" block.
 */


function block(array) {
  return wrap$1('{\n', indent(join(array, '\n')), '\n}');
}
/**
 * If maybeString is not null or empty, then wrap with start and end, otherwise print an empty string.
 */


function wrap$1(start, maybeString) {
  var end = arguments.length > 2 && arguments[2] !== undefined ? arguments[2] : '';
  return maybeString != null && maybeString !== '' ? start + maybeString + end : '';
}

function indent(str) {
  return wrap$1('  ', str.replace(/\n/g, '\n  '));
}

function isMultiline(str) {
  return str.indexOf('\n') !== -1;
}

function hasMultilineItems(maybeArray) {
  return maybeArray != null && maybeArray.some(isMultiline);
}

function isSelectionNode(node) {
  return node.kind === Kind$1.FIELD || node.kind === Kind$1.FRAGMENT_SPREAD || node.kind === Kind$1.INLINE_FRAGMENT;
}

function removeTemporaryGlobals() {
    return typeof Source$1 === "function" ? remove() : remove();
}

function checkDEV() {
    __DEV__ ? invariant$2("boolean" === typeof DEV, DEV) : invariant$2("boolean" === typeof DEV, 39);
}
removeTemporaryGlobals();
checkDEV();

function shouldInclude(_a, variables) {
    var directives = _a.directives;
    if (!directives || !directives.length) {
        return true;
    }
    return getInclusionDirectives(directives).every(function (_a) {
        var directive = _a.directive, ifArgument = _a.ifArgument;
        var evaledValue = false;
        if (ifArgument.value.kind === 'Variable') {
            evaledValue = variables && variables[ifArgument.value.name.value];
            __DEV__ ? invariant$2(evaledValue !== void 0, "Invalid variable referenced in @".concat(directive.name.value, " directive.")) : invariant$2(evaledValue !== void 0, 40);
        }
        else {
            evaledValue = ifArgument.value.value;
        }
        return directive.name.value === 'skip' ? !evaledValue : evaledValue;
    });
}
function hasDirectives(names, root, all) {
    var nameSet = new Set(names);
    var uniqueCount = nameSet.size;
    visit(root, {
        Directive: function (node) {
            if (nameSet.delete(node.name.value) &&
                (!all || !nameSet.size)) {
                return BREAK;
            }
        },
    });
    return all ? !nameSet.size : nameSet.size < uniqueCount;
}
function hasClientExports(document) {
    return document && hasDirectives(['client', 'export'], document, true);
}
function isInclusionDirective(_a) {
    var value = _a.name.value;
    return value === 'skip' || value === 'include';
}
function getInclusionDirectives(directives) {
    var result = [];
    if (directives && directives.length) {
        directives.forEach(function (directive) {
            if (!isInclusionDirective(directive))
                return;
            var directiveArguments = directive.arguments;
            var directiveName = directive.name.value;
            __DEV__ ? invariant$2(directiveArguments && directiveArguments.length === 1, "Incorrect number of arguments for the @".concat(directiveName, " directive.")) : invariant$2(directiveArguments && directiveArguments.length === 1, 41);
            var ifArgument = directiveArguments[0];
            __DEV__ ? invariant$2(ifArgument.name && ifArgument.name.value === 'if', "Invalid argument for the @".concat(directiveName, " directive.")) : invariant$2(ifArgument.name && ifArgument.name.value === 'if', 42);
            var ifValue = ifArgument.value;
            __DEV__ ? invariant$2(ifValue &&
                (ifValue.kind === 'Variable' || ifValue.kind === 'BooleanValue'), "Argument for the @".concat(directiveName, " directive must be a variable or a boolean value.")) : invariant$2(ifValue &&
                (ifValue.kind === 'Variable' || ifValue.kind === 'BooleanValue'), 43);
            result.push({ directive: directive, ifArgument: ifArgument });
        });
    }
    return result;
}

function getFragmentQueryDocument(document, fragmentName) {
    var actualFragmentName = fragmentName;
    var fragments = [];
    document.definitions.forEach(function (definition) {
        if (definition.kind === 'OperationDefinition') {
            throw __DEV__ ? new InvariantError("Found a ".concat(definition.operation, " operation").concat(definition.name ? " named '".concat(definition.name.value, "'") : '', ". ") +
                'No operations are allowed when using a fragment as a query. Only fragments are allowed.') : new InvariantError(44);
        }
        if (definition.kind === 'FragmentDefinition') {
            fragments.push(definition);
        }
    });
    if (typeof actualFragmentName === 'undefined') {
        __DEV__ ? invariant$2(fragments.length === 1, "Found ".concat(fragments.length, " fragments. `fragmentName` must be provided when there is not exactly 1 fragment.")) : invariant$2(fragments.length === 1, 45);
        actualFragmentName = fragments[0].name.value;
    }
    var query = __assign$2(__assign$2({}, document), { definitions: __spreadArray$1([
            {
                kind: 'OperationDefinition',
                operation: 'query',
                selectionSet: {
                    kind: 'SelectionSet',
                    selections: [
                        {
                            kind: 'FragmentSpread',
                            name: {
                                kind: 'Name',
                                value: actualFragmentName,
                            },
                        },
                    ],
                },
            }
        ], document.definitions, true) });
    return query;
}
function createFragmentMap(fragments) {
    if (fragments === void 0) { fragments = []; }
    var symTable = {};
    fragments.forEach(function (fragment) {
        symTable[fragment.name.value] = fragment;
    });
    return symTable;
}
function getFragmentFromSelection(selection, fragmentMap) {
    switch (selection.kind) {
        case 'InlineFragment':
            return selection;
        case 'FragmentSpread': {
            var fragmentName = selection.name.value;
            if (typeof fragmentMap === "function") {
                return fragmentMap(fragmentName);
            }
            var fragment = fragmentMap && fragmentMap[fragmentName];
            __DEV__ ? invariant$2(fragment, "No fragment named ".concat(fragmentName)) : invariant$2(fragment, 46);
            return fragment || null;
        }
        default:
            return null;
    }
}

function isNonNullObject(obj) {
    return obj !== null && typeof obj === 'object';
}

function makeReference(id) {
    return { __ref: String(id) };
}
function isReference(obj) {
    return Boolean(obj && typeof obj === 'object' && typeof obj.__ref === 'string');
}
function isDocumentNode(value) {
    return (isNonNullObject(value) &&
        value.kind === "Document" &&
        Array.isArray(value.definitions));
}
function isStringValue(value) {
    return value.kind === 'StringValue';
}
function isBooleanValue(value) {
    return value.kind === 'BooleanValue';
}
function isIntValue(value) {
    return value.kind === 'IntValue';
}
function isFloatValue(value) {
    return value.kind === 'FloatValue';
}
function isVariable(value) {
    return value.kind === 'Variable';
}
function isObjectValue(value) {
    return value.kind === 'ObjectValue';
}
function isListValue(value) {
    return value.kind === 'ListValue';
}
function isEnumValue(value) {
    return value.kind === 'EnumValue';
}
function isNullValue(value) {
    return value.kind === 'NullValue';
}
function valueToObjectRepresentation(argObj, name, value, variables) {
    if (isIntValue(value) || isFloatValue(value)) {
        argObj[name.value] = Number(value.value);
    }
    else if (isBooleanValue(value) || isStringValue(value)) {
        argObj[name.value] = value.value;
    }
    else if (isObjectValue(value)) {
        var nestedArgObj_1 = {};
        value.fields.map(function (obj) {
            return valueToObjectRepresentation(nestedArgObj_1, obj.name, obj.value, variables);
        });
        argObj[name.value] = nestedArgObj_1;
    }
    else if (isVariable(value)) {
        var variableValue = (variables || {})[value.name.value];
        argObj[name.value] = variableValue;
    }
    else if (isListValue(value)) {
        argObj[name.value] = value.values.map(function (listValue) {
            var nestedArgArrayObj = {};
            valueToObjectRepresentation(nestedArgArrayObj, name, listValue, variables);
            return nestedArgArrayObj[name.value];
        });
    }
    else if (isEnumValue(value)) {
        argObj[name.value] = value.value;
    }
    else if (isNullValue(value)) {
        argObj[name.value] = null;
    }
    else {
        throw __DEV__ ? new InvariantError("The inline argument \"".concat(name.value, "\" of kind \"").concat(value.kind, "\"") +
            'is not supported. Use variables instead of inline arguments to ' +
            'overcome this limitation.') : new InvariantError(55);
    }
}
function storeKeyNameFromField(field, variables) {
    var directivesObj = null;
    if (field.directives) {
        directivesObj = {};
        field.directives.forEach(function (directive) {
            directivesObj[directive.name.value] = {};
            if (directive.arguments) {
                directive.arguments.forEach(function (_a) {
                    var name = _a.name, value = _a.value;
                    return valueToObjectRepresentation(directivesObj[directive.name.value], name, value, variables);
                });
            }
        });
    }
    var argObj = null;
    if (field.arguments && field.arguments.length) {
        argObj = {};
        field.arguments.forEach(function (_a) {
            var name = _a.name, value = _a.value;
            return valueToObjectRepresentation(argObj, name, value, variables);
        });
    }
    return getStoreKeyName(field.name.value, argObj, directivesObj);
}
var KNOWN_DIRECTIVES = [
    'connection',
    'include',
    'skip',
    'client',
    'rest',
    'export',
];
var getStoreKeyName = Object.assign(function (fieldName, args, directives) {
    if (args &&
        directives &&
        directives['connection'] &&
        directives['connection']['key']) {
        if (directives['connection']['filter'] &&
            directives['connection']['filter'].length > 0) {
            var filterKeys = directives['connection']['filter']
                ? directives['connection']['filter']
                : [];
            filterKeys.sort();
            var filteredArgs_1 = {};
            filterKeys.forEach(function (key) {
                filteredArgs_1[key] = args[key];
            });
            return "".concat(directives['connection']['key'], "(").concat(stringify(filteredArgs_1), ")");
        }
        else {
            return directives['connection']['key'];
        }
    }
    var completeFieldName = fieldName;
    if (args) {
        var stringifiedArgs = stringify(args);
        completeFieldName += "(".concat(stringifiedArgs, ")");
    }
    if (directives) {
        Object.keys(directives).forEach(function (key) {
            if (KNOWN_DIRECTIVES.indexOf(key) !== -1)
                return;
            if (directives[key] && Object.keys(directives[key]).length) {
                completeFieldName += "@".concat(key, "(").concat(stringify(directives[key]), ")");
            }
            else {
                completeFieldName += "@".concat(key);
            }
        });
    }
    return completeFieldName;
}, {
    setStringify: function (s) {
        var previous = stringify;
        stringify = s;
        return previous;
    },
});
var stringify = function defaultStringify(value) {
    return JSON.stringify(value, stringifyReplacer);
};
function stringifyReplacer(_key, value) {
    if (isNonNullObject(value) && !Array.isArray(value)) {
        value = Object.keys(value).sort().reduce(function (copy, key) {
            copy[key] = value[key];
            return copy;
        }, {});
    }
    return value;
}
function argumentsObjectFromField(field, variables) {
    if (field.arguments && field.arguments.length) {
        var argObj_1 = {};
        field.arguments.forEach(function (_a) {
            var name = _a.name, value = _a.value;
            return valueToObjectRepresentation(argObj_1, name, value, variables);
        });
        return argObj_1;
    }
    return null;
}
function resultKeyNameFromField(field) {
    return field.alias ? field.alias.value : field.name.value;
}
function getTypenameFromResult(result, selectionSet, fragmentMap) {
    if (typeof result.__typename === 'string') {
        return result.__typename;
    }
    for (var _i = 0, _a = selectionSet.selections; _i < _a.length; _i++) {
        var selection = _a[_i];
        if (isField(selection)) {
            if (selection.name.value === '__typename') {
                return result[resultKeyNameFromField(selection)];
            }
        }
        else {
            var typename = getTypenameFromResult(result, getFragmentFromSelection(selection, fragmentMap).selectionSet, fragmentMap);
            if (typeof typename === 'string') {
                return typename;
            }
        }
    }
}
function isField(selection) {
    return selection.kind === 'Field';
}
function isInlineFragment(selection) {
    return selection.kind === 'InlineFragment';
}

function checkDocument(doc) {
    __DEV__ ? invariant$2(doc && doc.kind === 'Document', "Expecting a parsed GraphQL document. Perhaps you need to wrap the query string in a \"gql\" tag? http://docs.apollostack.com/apollo-client/core.html#gql") : invariant$2(doc && doc.kind === 'Document', 47);
    var operations = doc.definitions
        .filter(function (d) { return d.kind !== 'FragmentDefinition'; })
        .map(function (definition) {
        if (definition.kind !== 'OperationDefinition') {
            throw __DEV__ ? new InvariantError("Schema type definitions not allowed in queries. Found: \"".concat(definition.kind, "\"")) : new InvariantError(48);
        }
        return definition;
    });
    __DEV__ ? invariant$2(operations.length <= 1, "Ambiguous GraphQL document: contains ".concat(operations.length, " operations")) : invariant$2(operations.length <= 1, 49);
    return doc;
}
function getOperationDefinition(doc) {
    checkDocument(doc);
    return doc.definitions.filter(function (definition) {
        return definition.kind === 'OperationDefinition';
    })[0];
}
function getOperationName(doc) {
    return (doc.definitions
        .filter(function (definition) {
        return definition.kind === 'OperationDefinition' && !!definition.name;
    })
        .map(function (x) { return x.name.value; })[0] || null);
}
function getFragmentDefinitions(doc) {
    return doc.definitions.filter(function (definition) {
        return definition.kind === 'FragmentDefinition';
    });
}
function getQueryDefinition(doc) {
    var queryDef = getOperationDefinition(doc);
    __DEV__ ? invariant$2(queryDef && queryDef.operation === 'query', 'Must contain a query definition.') : invariant$2(queryDef && queryDef.operation === 'query', 50);
    return queryDef;
}
function getFragmentDefinition(doc) {
    __DEV__ ? invariant$2(doc.kind === 'Document', "Expecting a parsed GraphQL document. Perhaps you need to wrap the query string in a \"gql\" tag? http://docs.apollostack.com/apollo-client/core.html#gql") : invariant$2(doc.kind === 'Document', 51);
    __DEV__ ? invariant$2(doc.definitions.length <= 1, 'Fragment must have exactly one definition.') : invariant$2(doc.definitions.length <= 1, 52);
    var fragmentDef = doc.definitions[0];
    __DEV__ ? invariant$2(fragmentDef.kind === 'FragmentDefinition', 'Must be a fragment definition.') : invariant$2(fragmentDef.kind === 'FragmentDefinition', 53);
    return fragmentDef;
}
function getMainDefinition(queryDoc) {
    checkDocument(queryDoc);
    var fragmentDefinition;
    for (var _i = 0, _a = queryDoc.definitions; _i < _a.length; _i++) {
        var definition = _a[_i];
        if (definition.kind === 'OperationDefinition') {
            var operation = definition.operation;
            if (operation === 'query' ||
                operation === 'mutation' ||
                operation === 'subscription') {
                return definition;
            }
        }
        if (definition.kind === 'FragmentDefinition' && !fragmentDefinition) {
            fragmentDefinition = definition;
        }
    }
    if (fragmentDefinition) {
        return fragmentDefinition;
    }
    throw __DEV__ ? new InvariantError('Expected a parsed GraphQL query with a query, mutation, subscription, or a fragment.') : new InvariantError(54);
}
function getDefaultValues(definition) {
    var defaultValues = Object.create(null);
    var defs = definition && definition.variableDefinitions;
    if (defs && defs.length) {
        defs.forEach(function (def) {
            if (def.defaultValue) {
                valueToObjectRepresentation(defaultValues, def.variable.name, def.defaultValue);
            }
        });
    }
    return defaultValues;
}

var isArray = Array.isArray;
function isNonEmptyArray(value) {
    return Array.isArray(value) && value.length > 0;
}

var TYPENAME_FIELD = {
    kind: Kind$1.FIELD,
    name: {
        kind: Kind$1.NAME,
        value: '__typename',
    },
};
function isEmpty(op, fragmentMap) {
    return !op || op.selectionSet.selections.every(function (selection) { return selection.kind === Kind$1.FRAGMENT_SPREAD &&
        isEmpty(fragmentMap[selection.name.value], fragmentMap); });
}
function nullIfDocIsEmpty(doc) {
    return isEmpty(getOperationDefinition(doc) || getFragmentDefinition(doc), createFragmentMap(getFragmentDefinitions(doc)))
        ? null
        : doc;
}
function getDirectiveMatcher(directives) {
    var nameSet = new Set();
    var tests = [];
    directives.forEach(function (directive) {
        if (directive.name) {
            nameSet.add(directive.name);
        }
        else if (directive.test) {
            tests.push(directive.test);
        }
    });
    return function (directive) { return (nameSet.has(directive.name.value) ||
        tests.some(function (test) { return test(directive); })); };
}
function makeInUseGetterFunction(defaultKey) {
    var map = new Map();
    return function inUseGetterFunction(key) {
        if (key === void 0) { key = defaultKey; }
        var inUse = map.get(key);
        if (!inUse) {
            map.set(key, inUse = {
                variables: new Set,
                fragmentSpreads: new Set,
            });
        }
        return inUse;
    };
}
function removeDirectivesFromDocument(directives, doc) {
    var getInUseByOperationName = makeInUseGetterFunction("");
    var getInUseByFragmentName = makeInUseGetterFunction("");
    var getInUse = function (ancestors) {
        for (var p = 0, ancestor = void 0; p < ancestors.length && (ancestor = ancestors[p]); ++p) {
            if (isArray(ancestor))
                continue;
            if (ancestor.kind === Kind$1.OPERATION_DEFINITION) {
                return getInUseByOperationName(ancestor.name && ancestor.name.value);
            }
            if (ancestor.kind === Kind$1.FRAGMENT_DEFINITION) {
                return getInUseByFragmentName(ancestor.name.value);
            }
        }
        __DEV__ && invariant$2.error("Could not find operation or fragment");
        return null;
    };
    var operationCount = 0;
    for (var i = doc.definitions.length - 1; i >= 0; --i) {
        if (doc.definitions[i].kind === Kind$1.OPERATION_DEFINITION) {
            ++operationCount;
        }
    }
    var directiveMatcher = getDirectiveMatcher(directives);
    var hasRemoveDirective = directives.some(function (directive) { return directive.remove; });
    var shouldRemoveField = function (nodeDirectives) { return (hasRemoveDirective &&
        nodeDirectives &&
        nodeDirectives.some(directiveMatcher)); };
    var originalFragmentDefsByPath = new Map();
    var firstVisitMadeChanges = false;
    var fieldOrInlineFragmentVisitor = {
        enter: function (node) {
            if (shouldRemoveField(node.directives)) {
                firstVisitMadeChanges = true;
                return null;
            }
        },
    };
    var docWithoutDirectiveSubtrees = visit(doc, {
        Field: fieldOrInlineFragmentVisitor,
        InlineFragment: fieldOrInlineFragmentVisitor,
        VariableDefinition: {
            enter: function () {
                return false;
            },
        },
        Variable: {
            enter: function (node, _key, _parent, _path, ancestors) {
                var inUse = getInUse(ancestors);
                if (inUse) {
                    inUse.variables.add(node.name.value);
                }
            },
        },
        FragmentSpread: {
            enter: function (node, _key, _parent, _path, ancestors) {
                if (shouldRemoveField(node.directives)) {
                    firstVisitMadeChanges = true;
                    return null;
                }
                var inUse = getInUse(ancestors);
                if (inUse) {
                    inUse.fragmentSpreads.add(node.name.value);
                }
            },
        },
        FragmentDefinition: {
            enter: function (node, _key, _parent, path) {
                originalFragmentDefsByPath.set(JSON.stringify(path), node);
            },
            leave: function (node, _key, _parent, path) {
                var originalNode = originalFragmentDefsByPath.get(JSON.stringify(path));
                if (node === originalNode) {
                    return node;
                }
                if (operationCount > 0 &&
                    node.selectionSet.selections.every(function (selection) { return (selection.kind === Kind$1.FIELD &&
                        selection.name.value === '__typename'); })) {
                    getInUseByFragmentName(node.name.value).removed = true;
                    firstVisitMadeChanges = true;
                    return null;
                }
            },
        },
        Directive: {
            leave: function (node) {
                if (directiveMatcher(node)) {
                    firstVisitMadeChanges = true;
                    return null;
                }
            },
        },
    });
    if (!firstVisitMadeChanges) {
        return doc;
    }
    var populateTransitiveVars = function (inUse) {
        if (!inUse.transitiveVars) {
            inUse.transitiveVars = new Set(inUse.variables);
            if (!inUse.removed) {
                inUse.fragmentSpreads.forEach(function (childFragmentName) {
                    populateTransitiveVars(getInUseByFragmentName(childFragmentName)).transitiveVars.forEach(function (varName) {
                        inUse.transitiveVars.add(varName);
                    });
                });
            }
        }
        return inUse;
    };
    var allFragmentNamesUsed = new Set();
    docWithoutDirectiveSubtrees.definitions.forEach(function (def) {
        if (def.kind === Kind$1.OPERATION_DEFINITION) {
            populateTransitiveVars(getInUseByOperationName(def.name && def.name.value)).fragmentSpreads.forEach(function (childFragmentName) {
                allFragmentNamesUsed.add(childFragmentName);
            });
        }
        else if (def.kind === Kind$1.FRAGMENT_DEFINITION &&
            operationCount === 0 &&
            !getInUseByFragmentName(def.name.value).removed) {
            allFragmentNamesUsed.add(def.name.value);
        }
    });
    allFragmentNamesUsed.forEach(function (fragmentName) {
        populateTransitiveVars(getInUseByFragmentName(fragmentName)).fragmentSpreads.forEach(function (childFragmentName) {
            allFragmentNamesUsed.add(childFragmentName);
        });
    });
    var fragmentWillBeRemoved = function (fragmentName) { return !!(!allFragmentNamesUsed.has(fragmentName) ||
        getInUseByFragmentName(fragmentName).removed); };
    var enterVisitor = {
        enter: function (node) {
            if (fragmentWillBeRemoved(node.name.value)) {
                return null;
            }
        },
    };
    return nullIfDocIsEmpty(visit(docWithoutDirectiveSubtrees, {
        FragmentSpread: enterVisitor,
        FragmentDefinition: enterVisitor,
        OperationDefinition: {
            leave: function (node) {
                if (node.variableDefinitions) {
                    var usedVariableNames_1 = populateTransitiveVars(getInUseByOperationName(node.name && node.name.value)).transitiveVars;
                    if (usedVariableNames_1.size < node.variableDefinitions.length) {
                        return __assign$2(__assign$2({}, node), { variableDefinitions: node.variableDefinitions.filter(function (varDef) { return usedVariableNames_1.has(varDef.variable.name.value); }) });
                    }
                }
            },
        },
    }));
}
var addTypenameToDocument = Object.assign(function (doc) {
    return visit(doc, {
        SelectionSet: {
            enter: function (node, _key, parent) {
                if (parent &&
                    parent.kind === Kind$1.OPERATION_DEFINITION) {
                    return;
                }
                var selections = node.selections;
                if (!selections) {
                    return;
                }
                var skip = selections.some(function (selection) {
                    return (isField(selection) &&
                        (selection.name.value === '__typename' ||
                            selection.name.value.lastIndexOf('__', 0) === 0));
                });
                if (skip) {
                    return;
                }
                var field = parent;
                if (isField(field) &&
                    field.directives &&
                    field.directives.some(function (d) { return d.name.value === 'export'; })) {
                    return;
                }
                return __assign$2(__assign$2({}, node), { selections: __spreadArray$1(__spreadArray$1([], selections, true), [TYPENAME_FIELD], false) });
            },
        },
    });
}, {
    added: function (field) {
        return field === TYPENAME_FIELD;
    },
});
var connectionRemoveConfig = {
    test: function (directive) {
        var willRemove = directive.name.value === 'connection';
        if (willRemove) {
            if (!directive.arguments ||
                !directive.arguments.some(function (arg) { return arg.name.value === 'key'; })) {
                __DEV__ && invariant$2.warn('Removing an @connection directive even though it does not have a key. ' +
                    'You may want to use the key parameter to specify a store key.');
            }
        }
        return willRemove;
    },
};
function removeConnectionDirectiveFromDocument(doc) {
    return removeDirectivesFromDocument([connectionRemoveConfig], checkDocument(doc));
}
function buildQueryFromSelectionSet(document) {
    var definition = getMainDefinition(document);
    var definitionOperation = definition.operation;
    if (definitionOperation === 'query') {
        return document;
    }
    var modifiedDoc = visit(document, {
        OperationDefinition: {
            enter: function (node) {
                return __assign$2(__assign$2({}, node), { operation: 'query' });
            },
        },
    });
    return modifiedDoc;
}
function removeClientSetsFromDocument(document) {
    checkDocument(document);
    var modifiedDoc = removeDirectivesFromDocument([
        {
            test: function (directive) { return directive.name.value === 'client'; },
            remove: true,
        },
    ], document);
    return modifiedDoc;
}

var hasOwnProperty$5 = Object.prototype.hasOwnProperty;
function mergeDeep() {
    var sources = [];
    for (var _i = 0; _i < arguments.length; _i++) {
        sources[_i] = arguments[_i];
    }
    return mergeDeepArray(sources);
}
function mergeDeepArray(sources) {
    var target = sources[0] || {};
    var count = sources.length;
    if (count > 1) {
        var merger = new DeepMerger();
        for (var i = 1; i < count; ++i) {
            target = merger.merge(target, sources[i]);
        }
    }
    return target;
}
var defaultReconciler = function (target, source, property) {
    return this.merge(target[property], source[property]);
};
var DeepMerger = (function () {
    function DeepMerger(reconciler) {
        if (reconciler === void 0) { reconciler = defaultReconciler; }
        this.reconciler = reconciler;
        this.isObject = isNonNullObject;
        this.pastCopies = new Set();
    }
    DeepMerger.prototype.merge = function (target, source) {
        var _this = this;
        var context = [];
        for (var _i = 2; _i < arguments.length; _i++) {
            context[_i - 2] = arguments[_i];
        }
        if (isNonNullObject(source) && isNonNullObject(target)) {
            Object.keys(source).forEach(function (sourceKey) {
                if (hasOwnProperty$5.call(target, sourceKey)) {
                    var targetValue = target[sourceKey];
                    if (source[sourceKey] !== targetValue) {
                        var result = _this.reconciler.apply(_this, __spreadArray$1([target, source, sourceKey], context, false));
                        if (result !== targetValue) {
                            target = _this.shallowCopyForMerge(target);
                            target[sourceKey] = result;
                        }
                    }
                }
                else {
                    target = _this.shallowCopyForMerge(target);
                    target[sourceKey] = source[sourceKey];
                }
            });
            return target;
        }
        return source;
    };
    DeepMerger.prototype.shallowCopyForMerge = function (value) {
        if (isNonNullObject(value)) {
            if (!this.pastCopies.has(value)) {
                if (Array.isArray(value)) {
                    value = value.slice(0);
                }
                else {
                    value = __assign$2({ __proto__: Object.getPrototypeOf(value) }, value);
                }
                this.pastCopies.add(value);
            }
        }
        return value;
    };
    return DeepMerger;
}());

function _createForOfIteratorHelperLoose(o, allowArrayLike) { var it = typeof Symbol !== "undefined" && o[Symbol.iterator] || o["@@iterator"]; if (it) return (it = it.call(o)).next.bind(it); if (Array.isArray(o) || (it = _unsupportedIterableToArray(o)) || allowArrayLike && o && typeof o.length === "number") { if (it) o = it; var i = 0; return function () { if (i >= o.length) return { done: true }; return { done: false, value: o[i++] }; }; } throw new TypeError("Invalid attempt to iterate non-iterable instance.\nIn order to be iterable, non-array objects must have a [Symbol.iterator]() method."); }

function _unsupportedIterableToArray(o, minLen) { if (!o) return; if (typeof o === "string") return _arrayLikeToArray(o, minLen); var n = Object.prototype.toString.call(o).slice(8, -1); if (n === "Object" && o.constructor) n = o.constructor.name; if (n === "Map" || n === "Set") return Array.from(o); if (n === "Arguments" || /^(?:Ui|I)nt(?:8|16|32)(?:Clamped)?Array$/.test(n)) return _arrayLikeToArray(o, minLen); }

function _arrayLikeToArray(arr, len) { if (len == null || len > arr.length) len = arr.length; for (var i = 0, arr2 = new Array(len); i < len; i++) { arr2[i] = arr[i]; } return arr2; }

function _defineProperties$2(target, props) { for (var i = 0; i < props.length; i++) { var descriptor = props[i]; descriptor.enumerable = descriptor.enumerable || false; descriptor.configurable = true; if ("value" in descriptor) descriptor.writable = true; Object.defineProperty(target, descriptor.key, descriptor); } }

function _createClass$2(Constructor, protoProps, staticProps) { if (protoProps) _defineProperties$2(Constructor.prototype, protoProps); if (staticProps) _defineProperties$2(Constructor, staticProps); Object.defineProperty(Constructor, "prototype", { writable: false }); return Constructor; }

// === Symbol Support ===
var hasSymbols = function () {
  return typeof Symbol === 'function';
};

var hasSymbol = function (name) {
  return hasSymbols() && Boolean(Symbol[name]);
};

var getSymbol = function (name) {
  return hasSymbol(name) ? Symbol[name] : '@@' + name;
};

if (hasSymbols() && !hasSymbol('observable')) {
  Symbol.observable = Symbol('observable');
}

var SymbolIterator = getSymbol('iterator');
var SymbolObservable = getSymbol('observable');
var SymbolSpecies = getSymbol('species'); // === Abstract Operations ===

function getMethod(obj, key) {
  var value = obj[key];
  if (value == null) return undefined;
  if (typeof value !== 'function') throw new TypeError(value + ' is not a function');
  return value;
}

function getSpecies(obj) {
  var ctor = obj.constructor;

  if (ctor !== undefined) {
    ctor = ctor[SymbolSpecies];

    if (ctor === null) {
      ctor = undefined;
    }
  }

  return ctor !== undefined ? ctor : Observable;
}

function isObservable(x) {
  return x instanceof Observable; // SPEC: Brand check
}

function hostReportError(e) {
  if (hostReportError.log) {
    hostReportError.log(e);
  } else {
    setTimeout(function () {
      throw e;
    });
  }
}

function enqueue(fn) {
  Promise.resolve().then(function () {
    try {
      fn();
    } catch (e) {
      hostReportError(e);
    }
  });
}

function cleanupSubscription(subscription) {
  var cleanup = subscription._cleanup;
  if (cleanup === undefined) return;
  subscription._cleanup = undefined;

  if (!cleanup) {
    return;
  }

  try {
    if (typeof cleanup === 'function') {
      cleanup();
    } else {
      var unsubscribe = getMethod(cleanup, 'unsubscribe');

      if (unsubscribe) {
        unsubscribe.call(cleanup);
      }
    }
  } catch (e) {
    hostReportError(e);
  }
}

function closeSubscription(subscription) {
  subscription._observer = undefined;
  subscription._queue = undefined;
  subscription._state = 'closed';
}

function flushSubscription(subscription) {
  var queue = subscription._queue;

  if (!queue) {
    return;
  }

  subscription._queue = undefined;
  subscription._state = 'ready';

  for (var i = 0; i < queue.length; ++i) {
    notifySubscription(subscription, queue[i].type, queue[i].value);
    if (subscription._state === 'closed') break;
  }
}

function notifySubscription(subscription, type, value) {
  subscription._state = 'running';
  var observer = subscription._observer;

  try {
    var m = getMethod(observer, type);

    switch (type) {
      case 'next':
        if (m) m.call(observer, value);
        break;

      case 'error':
        closeSubscription(subscription);
        if (m) m.call(observer, value);else throw value;
        break;

      case 'complete':
        closeSubscription(subscription);
        if (m) m.call(observer);
        break;
    }
  } catch (e) {
    hostReportError(e);
  }

  if (subscription._state === 'closed') cleanupSubscription(subscription);else if (subscription._state === 'running') subscription._state = 'ready';
}

function onNotify(subscription, type, value) {
  if (subscription._state === 'closed') return;

  if (subscription._state === 'buffering') {
    subscription._queue.push({
      type: type,
      value: value
    });

    return;
  }

  if (subscription._state !== 'ready') {
    subscription._state = 'buffering';
    subscription._queue = [{
      type: type,
      value: value
    }];
    enqueue(function () {
      return flushSubscription(subscription);
    });
    return;
  }

  notifySubscription(subscription, type, value);
}

var Subscription = /*#__PURE__*/function () {
  function Subscription(observer, subscriber) {
    // ASSERT: observer is an object
    // ASSERT: subscriber is callable
    this._cleanup = undefined;
    this._observer = observer;
    this._queue = undefined;
    this._state = 'initializing';
    var subscriptionObserver = new SubscriptionObserver(this);

    try {
      this._cleanup = subscriber.call(undefined, subscriptionObserver);
    } catch (e) {
      subscriptionObserver.error(e);
    }

    if (this._state === 'initializing') this._state = 'ready';
  }

  var _proto = Subscription.prototype;

  _proto.unsubscribe = function unsubscribe() {
    if (this._state !== 'closed') {
      closeSubscription(this);
      cleanupSubscription(this);
    }
  };

  _createClass$2(Subscription, [{
    key: "closed",
    get: function () {
      return this._state === 'closed';
    }
  }]);

  return Subscription;
}();

var SubscriptionObserver = /*#__PURE__*/function () {
  function SubscriptionObserver(subscription) {
    this._subscription = subscription;
  }

  var _proto2 = SubscriptionObserver.prototype;

  _proto2.next = function next(value) {
    onNotify(this._subscription, 'next', value);
  };

  _proto2.error = function error(value) {
    onNotify(this._subscription, 'error', value);
  };

  _proto2.complete = function complete() {
    onNotify(this._subscription, 'complete');
  };

  _createClass$2(SubscriptionObserver, [{
    key: "closed",
    get: function () {
      return this._subscription._state === 'closed';
    }
  }]);

  return SubscriptionObserver;
}();

var Observable = /*#__PURE__*/function () {
  function Observable(subscriber) {
    if (!(this instanceof Observable)) throw new TypeError('Observable cannot be called as a function');
    if (typeof subscriber !== 'function') throw new TypeError('Observable initializer must be a function');
    this._subscriber = subscriber;
  }

  var _proto3 = Observable.prototype;

  _proto3.subscribe = function subscribe(observer) {
    if (typeof observer !== 'object' || observer === null) {
      observer = {
        next: observer,
        error: arguments[1],
        complete: arguments[2]
      };
    }

    return new Subscription(observer, this._subscriber);
  };

  _proto3.forEach = function forEach(fn) {
    var _this = this;

    return new Promise(function (resolve, reject) {
      if (typeof fn !== 'function') {
        reject(new TypeError(fn + ' is not a function'));
        return;
      }

      function done() {
        subscription.unsubscribe();
        resolve();
      }

      var subscription = _this.subscribe({
        next: function (value) {
          try {
            fn(value, done);
          } catch (e) {
            reject(e);
            subscription.unsubscribe();
          }
        },
        error: reject,
        complete: resolve
      });
    });
  };

  _proto3.map = function map(fn) {
    var _this2 = this;

    if (typeof fn !== 'function') throw new TypeError(fn + ' is not a function');
    var C = getSpecies(this);
    return new C(function (observer) {
      return _this2.subscribe({
        next: function (value) {
          try {
            value = fn(value);
          } catch (e) {
            return observer.error(e);
          }

          observer.next(value);
        },
        error: function (e) {
          observer.error(e);
        },
        complete: function () {
          observer.complete();
        }
      });
    });
  };

  _proto3.filter = function filter(fn) {
    var _this3 = this;

    if (typeof fn !== 'function') throw new TypeError(fn + ' is not a function');
    var C = getSpecies(this);
    return new C(function (observer) {
      return _this3.subscribe({
        next: function (value) {
          try {
            if (!fn(value)) return;
          } catch (e) {
            return observer.error(e);
          }

          observer.next(value);
        },
        error: function (e) {
          observer.error(e);
        },
        complete: function () {
          observer.complete();
        }
      });
    });
  };

  _proto3.reduce = function reduce(fn) {
    var _this4 = this;

    if (typeof fn !== 'function') throw new TypeError(fn + ' is not a function');
    var C = getSpecies(this);
    var hasSeed = arguments.length > 1;
    var hasValue = false;
    var seed = arguments[1];
    var acc = seed;
    return new C(function (observer) {
      return _this4.subscribe({
        next: function (value) {
          var first = !hasValue;
          hasValue = true;

          if (!first || hasSeed) {
            try {
              acc = fn(acc, value);
            } catch (e) {
              return observer.error(e);
            }
          } else {
            acc = value;
          }
        },
        error: function (e) {
          observer.error(e);
        },
        complete: function () {
          if (!hasValue && !hasSeed) return observer.error(new TypeError('Cannot reduce an empty sequence'));
          observer.next(acc);
          observer.complete();
        }
      });
    });
  };

  _proto3.concat = function concat() {
    var _this5 = this;

    for (var _len = arguments.length, sources = new Array(_len), _key = 0; _key < _len; _key++) {
      sources[_key] = arguments[_key];
    }

    var C = getSpecies(this);
    return new C(function (observer) {
      var subscription;
      var index = 0;

      function startNext(next) {
        subscription = next.subscribe({
          next: function (v) {
            observer.next(v);
          },
          error: function (e) {
            observer.error(e);
          },
          complete: function () {
            if (index === sources.length) {
              subscription = undefined;
              observer.complete();
            } else {
              startNext(C.from(sources[index++]));
            }
          }
        });
      }

      startNext(_this5);
      return function () {
        if (subscription) {
          subscription.unsubscribe();
          subscription = undefined;
        }
      };
    });
  };

  _proto3.flatMap = function flatMap(fn) {
    var _this6 = this;

    if (typeof fn !== 'function') throw new TypeError(fn + ' is not a function');
    var C = getSpecies(this);
    return new C(function (observer) {
      var subscriptions = [];

      var outer = _this6.subscribe({
        next: function (value) {
          if (fn) {
            try {
              value = fn(value);
            } catch (e) {
              return observer.error(e);
            }
          }

          var inner = C.from(value).subscribe({
            next: function (value) {
              observer.next(value);
            },
            error: function (e) {
              observer.error(e);
            },
            complete: function () {
              var i = subscriptions.indexOf(inner);
              if (i >= 0) subscriptions.splice(i, 1);
              completeIfDone();
            }
          });
          subscriptions.push(inner);
        },
        error: function (e) {
          observer.error(e);
        },
        complete: function () {
          completeIfDone();
        }
      });

      function completeIfDone() {
        if (outer.closed && subscriptions.length === 0) observer.complete();
      }

      return function () {
        subscriptions.forEach(function (s) {
          return s.unsubscribe();
        });
        outer.unsubscribe();
      };
    });
  };

  _proto3[SymbolObservable] = function () {
    return this;
  };

  Observable.from = function from(x) {
    var C = typeof this === 'function' ? this : Observable;
    if (x == null) throw new TypeError(x + ' is not an object');
    var method = getMethod(x, SymbolObservable);

    if (method) {
      var observable = method.call(x);
      if (Object(observable) !== observable) throw new TypeError(observable + ' is not an object');
      if (isObservable(observable) && observable.constructor === C) return observable;
      return new C(function (observer) {
        return observable.subscribe(observer);
      });
    }

    if (hasSymbol('iterator')) {
      method = getMethod(x, SymbolIterator);

      if (method) {
        return new C(function (observer) {
          enqueue(function () {
            if (observer.closed) return;

            for (var _iterator = _createForOfIteratorHelperLoose(method.call(x)), _step; !(_step = _iterator()).done;) {
              var item = _step.value;
              observer.next(item);
              if (observer.closed) return;
            }

            observer.complete();
          });
        });
      }
    }

    if (Array.isArray(x)) {
      return new C(function (observer) {
        enqueue(function () {
          if (observer.closed) return;

          for (var i = 0; i < x.length; ++i) {
            observer.next(x[i]);
            if (observer.closed) return;
          }

          observer.complete();
        });
      });
    }

    throw new TypeError(x + ' is not observable');
  };

  Observable.of = function of() {
    for (var _len2 = arguments.length, items = new Array(_len2), _key2 = 0; _key2 < _len2; _key2++) {
      items[_key2] = arguments[_key2];
    }

    var C = typeof this === 'function' ? this : Observable;
    return new C(function (observer) {
      enqueue(function () {
        if (observer.closed) return;

        for (var i = 0; i < items.length; ++i) {
          observer.next(items[i]);
          if (observer.closed) return;
        }

        observer.complete();
      });
    });
  };

  _createClass$2(Observable, null, [{
    key: SymbolSpecies,
    get: function () {
      return this;
    }
  }]);

  return Observable;
}();

if (hasSymbols()) {
  Object.defineProperty(Observable, Symbol('extensions'), {
    value: {
      symbol: SymbolObservable,
      hostReportError: hostReportError
    },
    configurable: true
  });
}

function symbolObservablePonyfill$1(root) {
	var result;
	var Symbol = root.Symbol;

	if (typeof Symbol === 'function') {
		if (Symbol.observable) {
			result = Symbol.observable;
		} else {

			if (typeof Symbol.for === 'function') {
				// This just needs to be something that won't trample other user's Symbol.for use
				// It also will guide people to the source of their issues, if this is problematic.
				// META: It's a resource locator!
				result = Symbol.for('https://github.com/benlesh/symbol-observable');
			} else {
				// Symbol.for didn't exist! The best we can do at this point is a totally 
				// unique symbol. Note that the string argument here is a descriptor, not
				// an identifier. This symbol is unique.
				result = Symbol('https://github.com/benlesh/symbol-observable');
			}
			try {
				Symbol.observable = result;
			} catch (err) {
				// Do nothing. In some environments, users have frozen `Symbol` for security reasons,
				// if it is frozen assigning to it will throw. In this case, we don't care, because
				// they will need to use the returned value from the ponyfill.
			}
		}
	} else {
		result = '@@observable';
	}

	return result;
}

/* global window */

var root$1;

if (typeof self !== 'undefined') {
  root$1 = self;
} else if (typeof window !== 'undefined') {
  root$1 = window;
} else if (typeof global !== 'undefined') {
  root$1 = global;
} else if (typeof module !== 'undefined') {
  root$1 = module;
} else {
  root$1 = Function('return this')();
}

symbolObservablePonyfill$1(root$1);

var prototype = Observable.prototype;
var fakeObsSymbol = '@@observable';
if (!prototype[fakeObsSymbol]) {
    prototype[fakeObsSymbol] = function () { return this; };
}

var toString$1 = Object.prototype.toString;
function cloneDeep(value) {
    return cloneDeepHelper(value);
}
function cloneDeepHelper(val, seen) {
    switch (toString$1.call(val)) {
        case "[object Array]": {
            seen = seen || new Map;
            if (seen.has(val))
                return seen.get(val);
            var copy_1 = val.slice(0);
            seen.set(val, copy_1);
            copy_1.forEach(function (child, i) {
                copy_1[i] = cloneDeepHelper(child, seen);
            });
            return copy_1;
        }
        case "[object Object]": {
            seen = seen || new Map;
            if (seen.has(val))
                return seen.get(val);
            var copy_2 = Object.create(Object.getPrototypeOf(val));
            seen.set(val, copy_2);
            Object.keys(val).forEach(function (key) {
                copy_2[key] = cloneDeepHelper(val[key], seen);
            });
            return copy_2;
        }
        default:
            return val;
    }
}

function deepFreeze(value) {
    var workSet = new Set([value]);
    workSet.forEach(function (obj) {
        if (isNonNullObject(obj) && shallowFreeze(obj) === obj) {
            Object.getOwnPropertyNames(obj).forEach(function (name) {
                if (isNonNullObject(obj[name]))
                    workSet.add(obj[name]);
            });
        }
    });
    return value;
}
function shallowFreeze(obj) {
    if (__DEV__ && !Object.isFrozen(obj)) {
        try {
            Object.freeze(obj);
        }
        catch (e) {
            if (e instanceof TypeError)
                return null;
            throw e;
        }
    }
    return obj;
}
function maybeDeepFreeze(obj) {
    if (__DEV__) {
        deepFreeze(obj);
    }
    return obj;
}

function iterateObserversSafely(observers, method, argument) {
    var observersWithMethod = [];
    observers.forEach(function (obs) { return obs[method] && observersWithMethod.push(obs); });
    observersWithMethod.forEach(function (obs) { return obs[method](argument); });
}

function asyncMap(observable, mapFn, catchFn) {
    return new Observable(function (observer) {
        var next = observer.next, error = observer.error, complete = observer.complete;
        var activeCallbackCount = 0;
        var completed = false;
        var promiseQueue = {
            then: function (callback) {
                return new Promise(function (resolve) { return resolve(callback()); });
            },
        };
        function makeCallback(examiner, delegate) {
            if (examiner) {
                return function (arg) {
                    ++activeCallbackCount;
                    var both = function () { return examiner(arg); };
                    promiseQueue = promiseQueue.then(both, both).then(function (result) {
                        --activeCallbackCount;
                        next && next.call(observer, result);
                        if (completed) {
                            handler.complete();
                        }
                    }, function (error) {
                        --activeCallbackCount;
                        throw error;
                    }).catch(function (caught) {
                        error && error.call(observer, caught);
                    });
                };
            }
            else {
                return function (arg) { return delegate && delegate.call(observer, arg); };
            }
        }
        var handler = {
            next: makeCallback(mapFn, next),
            error: makeCallback(catchFn, error),
            complete: function () {
                completed = true;
                if (!activeCallbackCount) {
                    complete && complete.call(observer);
                }
            },
        };
        var sub = observable.subscribe(handler);
        return function () { return sub.unsubscribe(); };
    });
}

var canUseWeakMap = typeof WeakMap === 'function' &&
    maybe$2(function () { return navigator.product; }) !== 'ReactNative';
var canUseWeakSet = typeof WeakSet === 'function';
var canUseSymbol = typeof Symbol === 'function' &&
    typeof Symbol.for === 'function';
var canUseAsyncIteratorSymbol = canUseSymbol && Symbol.asyncIterator;
typeof maybe$2(function () { return window.document.createElement; }) === "function";
maybe$2(function () { return navigator.userAgent.indexOf("jsdom") >= 0; }) || false;

function fixObservableSubclass(subclass) {
    function set(key) {
        Object.defineProperty(subclass, key, { value: Observable });
    }
    if (canUseSymbol && Symbol.species) {
        set(Symbol.species);
    }
    set("@@species");
    return subclass;
}

function isPromiseLike(value) {
    return value && typeof value.then === "function";
}
var Concast = (function (_super) {
    __extends(Concast, _super);
    function Concast(sources) {
        var _this = _super.call(this, function (observer) {
            _this.addObserver(observer);
            return function () { return _this.removeObserver(observer); };
        }) || this;
        _this.observers = new Set();
        _this.promise = new Promise(function (resolve, reject) {
            _this.resolve = resolve;
            _this.reject = reject;
        });
        _this.handlers = {
            next: function (result) {
                if (_this.sub !== null) {
                    _this.latest = ["next", result];
                    _this.notify("next", result);
                    iterateObserversSafely(_this.observers, "next", result);
                }
            },
            error: function (error) {
                var sub = _this.sub;
                if (sub !== null) {
                    if (sub)
                        setTimeout(function () { return sub.unsubscribe(); });
                    _this.sub = null;
                    _this.latest = ["error", error];
                    _this.reject(error);
                    _this.notify("error", error);
                    iterateObserversSafely(_this.observers, "error", error);
                }
            },
            complete: function () {
                var _a = _this, sub = _a.sub, _b = _a.sources, sources = _b === void 0 ? [] : _b;
                if (sub !== null) {
                    var value = sources.shift();
                    if (!value) {
                        if (sub)
                            setTimeout(function () { return sub.unsubscribe(); });
                        _this.sub = null;
                        if (_this.latest &&
                            _this.latest[0] === "next") {
                            _this.resolve(_this.latest[1]);
                        }
                        else {
                            _this.resolve();
                        }
                        _this.notify("complete");
                        iterateObserversSafely(_this.observers, "complete");
                    }
                    else if (isPromiseLike(value)) {
                        value.then(function (obs) { return _this.sub = obs.subscribe(_this.handlers); });
                    }
                    else {
                        _this.sub = value.subscribe(_this.handlers);
                    }
                }
            },
        };
        _this.nextResultListeners = new Set();
        _this.cancel = function (reason) {
            _this.reject(reason);
            _this.sources = [];
            _this.handlers.complete();
        };
        _this.promise.catch(function (_) { });
        if (typeof sources === "function") {
            sources = [new Observable(sources)];
        }
        if (isPromiseLike(sources)) {
            sources.then(function (iterable) { return _this.start(iterable); }, _this.handlers.error);
        }
        else {
            _this.start(sources);
        }
        return _this;
    }
    Concast.prototype.start = function (sources) {
        if (this.sub !== void 0)
            return;
        this.sources = Array.from(sources);
        this.handlers.complete();
    };
    Concast.prototype.deliverLastMessage = function (observer) {
        if (this.latest) {
            var nextOrError = this.latest[0];
            var method = observer[nextOrError];
            if (method) {
                method.call(observer, this.latest[1]);
            }
            if (this.sub === null &&
                nextOrError === "next" &&
                observer.complete) {
                observer.complete();
            }
        }
    };
    Concast.prototype.addObserver = function (observer) {
        if (!this.observers.has(observer)) {
            this.deliverLastMessage(observer);
            this.observers.add(observer);
        }
    };
    Concast.prototype.removeObserver = function (observer) {
        if (this.observers.delete(observer) &&
            this.observers.size < 1) {
            this.handlers.complete();
        }
    };
    Concast.prototype.notify = function (method, arg) {
        var nextResultListeners = this.nextResultListeners;
        if (nextResultListeners.size) {
            this.nextResultListeners = new Set;
            nextResultListeners.forEach(function (listener) { return listener(method, arg); });
        }
    };
    Concast.prototype.beforeNext = function (callback) {
        var called = false;
        this.nextResultListeners.add(function (method, arg) {
            if (!called) {
                called = true;
                callback(method, arg);
            }
        });
    };
    return Concast;
}(Observable));
fixObservableSubclass(Concast);

function isExecutionPatchIncrementalResult(value) {
    return "incremental" in value;
}
function isExecutionPatchInitialResult(value) {
    return "hasNext" in value && "data" in value;
}
function isExecutionPatchResult(value) {
    return (isExecutionPatchIncrementalResult(value) ||
        isExecutionPatchInitialResult(value));
}
function mergeIncrementalData(prevResult, result) {
    var mergedData = prevResult;
    var merger = new DeepMerger();
    if (isExecutionPatchIncrementalResult(result) &&
        isNonEmptyArray(result.incremental)) {
        result.incremental.forEach(function (_a) {
            var data = _a.data, path = _a.path;
            for (var i = path.length - 1; i >= 0; --i) {
                var key = path[i];
                var isNumericKey = !isNaN(+key);
                var parent_1 = isNumericKey ? [] : {};
                parent_1[key] = data;
                data = parent_1;
            }
            mergedData = merger.merge(mergedData, data);
        });
    }
    return mergedData;
}

function graphQLResultHasError(result) {
    var errors = getGraphQLErrorsFromResult(result);
    return isNonEmptyArray(errors);
}
function getGraphQLErrorsFromResult(result) {
    var graphQLErrors = isNonEmptyArray(result.errors)
        ? result.errors.slice(0)
        : [];
    if (isExecutionPatchIncrementalResult(result) &&
        isNonEmptyArray(result.incremental)) {
        result.incremental.forEach(function (incrementalResult) {
            if (incrementalResult.errors) {
                graphQLErrors.push.apply(graphQLErrors, incrementalResult.errors);
            }
        });
    }
    return graphQLErrors;
}

function compact() {
    var objects = [];
    for (var _i = 0; _i < arguments.length; _i++) {
        objects[_i] = arguments[_i];
    }
    var result = Object.create(null);
    objects.forEach(function (obj) {
        if (!obj)
            return;
        Object.keys(obj).forEach(function (key) {
            var value = obj[key];
            if (value !== void 0) {
                result[key] = value;
            }
        });
    });
    return result;
}

var prefixCounts = new Map();
function makeUniqueId(prefix) {
    var count = prefixCounts.get(prefix) || 1;
    prefixCounts.set(prefix, count + 1);
    return "".concat(prefix, ":").concat(count, ":").concat(Math.random().toString(36).slice(2));
}

function stringifyForDisplay(value) {
    var undefId = makeUniqueId("stringifyForDisplay");
    return JSON.stringify(value, function (key, value) {
        return value === void 0 ? undefId : value;
    }).split(JSON.stringify(undefId)).join("<undefined>");
}

function mergeOptions(defaults, options) {
    return compact(defaults, options, options.variables && {
        variables: __assign$2(__assign$2({}, (defaults && defaults.variables)), options.variables),
    });
}

function fromError(errorValue) {
    return new Observable(function (observer) {
        observer.error(errorValue);
    });
}

var throwServerError = function (response, result, message) {
    var error = new Error(message);
    error.name = 'ServerError';
    error.response = response;
    error.statusCode = response.status;
    error.result = result;
    throw error;
};

function validateOperation(operation) {
    var OPERATION_FIELDS = [
        'query',
        'operationName',
        'variables',
        'extensions',
        'context',
    ];
    for (var _i = 0, _a = Object.keys(operation); _i < _a.length; _i++) {
        var key = _a[_i];
        if (OPERATION_FIELDS.indexOf(key) < 0) {
            throw __DEV__ ? new InvariantError("illegal argument: ".concat(key)) : new InvariantError(27);
        }
    }
    return operation;
}

function createOperation(starting, operation) {
    var context = __assign$2({}, starting);
    var setContext = function (next) {
        if (typeof next === 'function') {
            context = __assign$2(__assign$2({}, context), next(context));
        }
        else {
            context = __assign$2(__assign$2({}, context), next);
        }
    };
    var getContext = function () { return (__assign$2({}, context)); };
    Object.defineProperty(operation, 'setContext', {
        enumerable: false,
        value: setContext,
    });
    Object.defineProperty(operation, 'getContext', {
        enumerable: false,
        value: getContext,
    });
    return operation;
}

function transformOperation(operation) {
    var transformedOperation = {
        variables: operation.variables || {},
        extensions: operation.extensions || {},
        operationName: operation.operationName,
        query: operation.query,
    };
    if (!transformedOperation.operationName) {
        transformedOperation.operationName =
            typeof transformedOperation.query !== 'string'
                ? getOperationName(transformedOperation.query) || undefined
                : '';
    }
    return transformedOperation;
}

function passthrough(op, forward) {
    return (forward ? forward(op) : Observable.of());
}
function toLink(handler) {
    return typeof handler === 'function' ? new ApolloLink(handler) : handler;
}
function isTerminating(link) {
    return link.request.length <= 1;
}
var LinkError = (function (_super) {
    __extends(LinkError, _super);
    function LinkError(message, link) {
        var _this = _super.call(this, message) || this;
        _this.link = link;
        return _this;
    }
    return LinkError;
}(Error));
var ApolloLink = (function () {
    function ApolloLink(request) {
        if (request)
            this.request = request;
    }
    ApolloLink.empty = function () {
        return new ApolloLink(function () { return Observable.of(); });
    };
    ApolloLink.from = function (links) {
        if (links.length === 0)
            return ApolloLink.empty();
        return links.map(toLink).reduce(function (x, y) { return x.concat(y); });
    };
    ApolloLink.split = function (test, left, right) {
        var leftLink = toLink(left);
        var rightLink = toLink(right || new ApolloLink(passthrough));
        if (isTerminating(leftLink) && isTerminating(rightLink)) {
            return new ApolloLink(function (operation) {
                return test(operation)
                    ? leftLink.request(operation) || Observable.of()
                    : rightLink.request(operation) || Observable.of();
            });
        }
        else {
            return new ApolloLink(function (operation, forward) {
                return test(operation)
                    ? leftLink.request(operation, forward) || Observable.of()
                    : rightLink.request(operation, forward) || Observable.of();
            });
        }
    };
    ApolloLink.execute = function (link, operation) {
        return (link.request(createOperation(operation.context, transformOperation(validateOperation(operation)))) || Observable.of());
    };
    ApolloLink.concat = function (first, second) {
        var firstLink = toLink(first);
        if (isTerminating(firstLink)) {
            __DEV__ && invariant$2.warn(new LinkError("You are calling concat on a terminating link, which will have no effect", firstLink));
            return firstLink;
        }
        var nextLink = toLink(second);
        if (isTerminating(nextLink)) {
            return new ApolloLink(function (operation) {
                return firstLink.request(operation, function (op) { return nextLink.request(op) || Observable.of(); }) || Observable.of();
            });
        }
        else {
            return new ApolloLink(function (operation, forward) {
                return (firstLink.request(operation, function (op) {
                    return nextLink.request(op, forward) || Observable.of();
                }) || Observable.of());
            });
        }
    };
    ApolloLink.prototype.split = function (test, left, right) {
        return this.concat(ApolloLink.split(test, left, right || new ApolloLink(passthrough)));
    };
    ApolloLink.prototype.concat = function (next) {
        return ApolloLink.concat(this, next);
    };
    ApolloLink.prototype.request = function (operation, forward) {
        throw __DEV__ ? new InvariantError('request is not implemented') : new InvariantError(22);
    };
    ApolloLink.prototype.onError = function (error, observer) {
        if (observer && observer.error) {
            observer.error(error);
            return false;
        }
        throw error;
    };
    ApolloLink.prototype.setOnError = function (fn) {
        this.onError = fn;
        return this;
    };
    return ApolloLink;
}());

var execute = ApolloLink.execute;

var version = '3.7.10';

function isNodeResponse(value) {
    return !!value.body;
}
function isReadableStream(value) {
    return !!value.getReader;
}
function isAsyncIterableIterator(value) {
    return !!(canUseAsyncIteratorSymbol &&
        value[Symbol.asyncIterator]);
}
function isStreamableBlob(value) {
    return !!value.stream;
}
function isBlob(value) {
    return !!value.arrayBuffer;
}
function isNodeReadableStream(value) {
    return !!value.pipe;
}

function asyncIterator(source) {
    var _a;
    var iterator = source[Symbol.asyncIterator]();
    return _a = {
            next: function () {
                return iterator.next();
            }
        },
        _a[Symbol.asyncIterator] = function () {
            return this;
        },
        _a;
}

function nodeStreamIterator(stream) {
    var cleanup = null;
    var error = null;
    var done = false;
    var data = [];
    var waiting = [];
    function onData(chunk) {
        if (error)
            return;
        if (waiting.length) {
            var shiftedArr = waiting.shift();
            if (Array.isArray(shiftedArr) && shiftedArr[0]) {
                return shiftedArr[0]({ value: chunk, done: false });
            }
        }
        data.push(chunk);
    }
    function onError(err) {
        error = err;
        var all = waiting.slice();
        all.forEach(function (pair) {
            pair[1](err);
        });
        !cleanup || cleanup();
    }
    function onEnd() {
        done = true;
        var all = waiting.slice();
        all.forEach(function (pair) {
            pair[0]({ value: undefined, done: true });
        });
        !cleanup || cleanup();
    }
    cleanup = function () {
        cleanup = null;
        stream.removeListener("data", onData);
        stream.removeListener("error", onError);
        stream.removeListener("end", onEnd);
        stream.removeListener("finish", onEnd);
        stream.removeListener("close", onEnd);
    };
    stream.on("data", onData);
    stream.on("error", onError);
    stream.on("end", onEnd);
    stream.on("finish", onEnd);
    stream.on("close", onEnd);
    function getNext() {
        return new Promise(function (resolve, reject) {
            if (error)
                return reject(error);
            if (data.length)
                return resolve({ value: data.shift(), done: false });
            if (done)
                return resolve({ value: undefined, done: true });
            waiting.push([resolve, reject]);
        });
    }
    var iterator = {
        next: function () {
            return getNext();
        },
    };
    if (canUseAsyncIteratorSymbol) {
        iterator[Symbol.asyncIterator] = function () {
            return this;
        };
    }
    return iterator;
}

function promiseIterator(promise) {
    var resolved = false;
    var iterator = {
        next: function () {
            if (resolved)
                return Promise.resolve({
                    value: undefined,
                    done: true,
                });
            resolved = true;
            return new Promise(function (resolve, reject) {
                promise
                    .then(function (value) {
                    resolve({ value: value, done: false });
                })
                    .catch(reject);
            });
        },
    };
    if (canUseAsyncIteratorSymbol) {
        iterator[Symbol.asyncIterator] = function () {
            return this;
        };
    }
    return iterator;
}

function readerIterator(reader) {
    var iterator = {
        next: function () {
            return reader.read();
        },
    };
    if (canUseAsyncIteratorSymbol) {
        iterator[Symbol.asyncIterator] = function () {
            return this;
        };
    }
    return iterator;
}

function responseIterator(response) {
    var body = response;
    if (isNodeResponse(response))
        body = response.body;
    if (isAsyncIterableIterator(body))
        return asyncIterator(body);
    if (isReadableStream(body))
        return readerIterator(body.getReader());
    if (isStreamableBlob(body)) {
        return readerIterator(body.stream().getReader());
    }
    if (isBlob(body))
        return promiseIterator(body.arrayBuffer());
    if (isNodeReadableStream(body))
        return nodeStreamIterator(body);
    throw new Error("Unknown body type for responseIterator. Please pass a streamable response.");
}

var hasOwnProperty$4 = Object.prototype.hasOwnProperty;
function readMultipartBody(response, observer) {
    var _a, _b, _c;
    return __awaiter$1(this, void 0, void 0, function () {
        var decoder, contentType, delimiter, boundaryVal, boundary, buffer, iterator, running, _d, value, done, chunk, bi, message, i, headers, contentType_1, body, result;
        var _e;
        return __generator$1(this, function (_f) {
            switch (_f.label) {
                case 0:
                    if (TextDecoder === undefined) {
                        throw new Error("TextDecoder must be defined in the environment: please import a polyfill.");
                    }
                    decoder = new TextDecoder("utf-8");
                    contentType = (_a = response.headers) === null || _a === void 0 ? void 0 : _a.get('content-type');
                    delimiter = "boundary=";
                    boundaryVal = (contentType === null || contentType === void 0 ? void 0 : contentType.includes(delimiter))
                        ? contentType === null || contentType === void 0 ? void 0 : contentType.substring((contentType === null || contentType === void 0 ? void 0 : contentType.indexOf(delimiter)) + delimiter.length).replace(/['"]/g, "").replace(/\;(.*)/gm, "").trim()
                        : "-";
                    boundary = "--".concat(boundaryVal);
                    buffer = "";
                    iterator = responseIterator(response);
                    running = true;
                    _f.label = 1;
                case 1:
                    if (!running) return [3, 3];
                    return [4, iterator.next()];
                case 2:
                    _d = _f.sent(), value = _d.value, done = _d.done;
                    chunk = typeof value === "string" ? value : decoder.decode(value);
                    running = !done;
                    buffer += chunk;
                    bi = buffer.indexOf(boundary);
                    while (bi > -1) {
                        message = void 0;
                        _e = [
                            buffer.slice(0, bi),
                            buffer.slice(bi + boundary.length),
                        ], message = _e[0], buffer = _e[1];
                        if (message.trim()) {
                            i = message.indexOf("\r\n\r\n");
                            headers = parseHeaders(message.slice(0, i));
                            contentType_1 = headers["content-type"];
                            if (contentType_1 &&
                                contentType_1.toLowerCase().indexOf("application/json") === -1) {
                                throw new Error("Unsupported patch content type: application/json is required.");
                            }
                            body = message.slice(i);
                            try {
                                result = parseJsonBody(response, body.replace("\r\n", ""));
                                if (Object.keys(result).length > 1 ||
                                    "data" in result ||
                                    "incremental" in result ||
                                    "errors" in result) {
                                    (_b = observer.next) === null || _b === void 0 ? void 0 : _b.call(observer, result);
                                }
                            }
                            catch (err) {
                                handleError(err, observer);
                            }
                        }
                        bi = buffer.indexOf(boundary);
                    }
                    return [3, 1];
                case 3:
                    (_c = observer.complete) === null || _c === void 0 ? void 0 : _c.call(observer);
                    return [2];
            }
        });
    });
}
function parseHeaders(headerText) {
    var headersInit = {};
    headerText.split("\n").forEach(function (line) {
        var i = line.indexOf(":");
        if (i > -1) {
            var name_1 = line.slice(0, i).trim().toLowerCase();
            var value = line.slice(i + 1).trim();
            headersInit[name_1] = value;
        }
    });
    return headersInit;
}
function parseJsonBody(response, bodyText) {
    if (response.status >= 300) {
        var getResult = function () {
            try {
                return JSON.parse(bodyText);
            }
            catch (err) {
                return bodyText;
            }
        };
        throwServerError(response, getResult(), "Response not successful: Received status code ".concat(response.status));
    }
    try {
        return JSON.parse(bodyText);
    }
    catch (err) {
        var parseError = err;
        parseError.name = "ServerParseError";
        parseError.response = response;
        parseError.statusCode = response.status;
        parseError.bodyText = bodyText;
        throw parseError;
    }
}
function handleError(err, observer) {
    var _a, _b;
    if (err.name === "AbortError")
        return;
    if (err.result && err.result.errors && err.result.data) {
        (_a = observer.next) === null || _a === void 0 ? void 0 : _a.call(observer, err.result);
    }
    (_b = observer.error) === null || _b === void 0 ? void 0 : _b.call(observer, err);
}
function readJsonBody(response, operation, observer) {
    parseAndCheckHttpResponse(operation)(response)
        .then(function (result) {
        var _a, _b;
        (_a = observer.next) === null || _a === void 0 ? void 0 : _a.call(observer, result);
        (_b = observer.complete) === null || _b === void 0 ? void 0 : _b.call(observer);
    })
        .catch(function (err) { return handleError(err, observer); });
}
function parseAndCheckHttpResponse(operations) {
    return function (response) {
        return response
            .text()
            .then(function (bodyText) { return parseJsonBody(response, bodyText); })
            .then(function (result) {
            if (response.status >= 300) {
                throwServerError(response, result, "Response not successful: Received status code ".concat(response.status));
            }
            if (!Array.isArray(result) &&
                !hasOwnProperty$4.call(result, "data") &&
                !hasOwnProperty$4.call(result, "errors")) {
                throwServerError(response, result, "Server response was missing for query '".concat(Array.isArray(operations)
                    ? operations.map(function (op) { return op.operationName; })
                    : operations.operationName, "'."));
            }
            return result;
        });
    };
}

var serializeFetchParameter = function (p, label) {
    var serialized;
    try {
        serialized = JSON.stringify(p);
    }
    catch (e) {
        var parseError = __DEV__ ? new InvariantError("Network request failed. ".concat(label, " is not serializable: ").concat(e.message)) : new InvariantError(24);
        parseError.parseError = e;
        throw parseError;
    }
    return serialized;
};

var defaultHttpOptions = {
    includeQuery: true,
    includeExtensions: false,
    preserveHeaderCase: false,
};
var defaultHeaders = {
    accept: '*/*',
    'content-type': 'application/json',
};
var defaultOptions = {
    method: 'POST',
};
var fallbackHttpConfig = {
    http: defaultHttpOptions,
    headers: defaultHeaders,
    options: defaultOptions,
};
var defaultPrinter = function (ast, printer) { return printer(ast); };
function selectHttpOptionsAndBodyInternal(operation, printer) {
    var configs = [];
    for (var _i = 2; _i < arguments.length; _i++) {
        configs[_i - 2] = arguments[_i];
    }
    var options = {};
    var http = {};
    configs.forEach(function (config) {
        options = __assign$2(__assign$2(__assign$2({}, options), config.options), { headers: __assign$2(__assign$2({}, options.headers), config.headers) });
        if (config.credentials) {
            options.credentials = config.credentials;
        }
        http = __assign$2(__assign$2({}, http), config.http);
    });
    if (options.headers) {
        options.headers = removeDuplicateHeaders(options.headers, http.preserveHeaderCase);
    }
    var operationName = operation.operationName, extensions = operation.extensions, variables = operation.variables, query = operation.query;
    var body = { operationName: operationName, variables: variables };
    if (http.includeExtensions)
        body.extensions = extensions;
    if (http.includeQuery)
        body.query = printer(query, print);
    return {
        options: options,
        body: body,
    };
}
function removeDuplicateHeaders(headers, preserveHeaderCase) {
    if (!preserveHeaderCase) {
        var normalizedHeaders_1 = Object.create(null);
        Object.keys(Object(headers)).forEach(function (name) {
            normalizedHeaders_1[name.toLowerCase()] = headers[name];
        });
        return normalizedHeaders_1;
    }
    var headerData = Object.create(null);
    Object.keys(Object(headers)).forEach(function (name) {
        headerData[name.toLowerCase()] = { originalName: name, value: headers[name] };
    });
    var normalizedHeaders = Object.create(null);
    Object.keys(headerData).forEach(function (name) {
        normalizedHeaders[headerData[name].originalName] = headerData[name].value;
    });
    return normalizedHeaders;
}

var checkFetcher = function (fetcher) {
    if (!fetcher && typeof fetch === 'undefined') {
        throw __DEV__ ? new InvariantError("\n\"fetch\" has not been found globally and no fetcher has been configured. To fix this, install a fetch package (like https://www.npmjs.com/package/cross-fetch), instantiate the fetcher, and pass it into your HttpLink constructor. For example:\n\nimport fetch from 'cross-fetch';\nimport { ApolloClient, HttpLink } from '@apollo/client';\nconst client = new ApolloClient({\n  link: new HttpLink({ uri: '/graphql', fetch })\n});\n    ") : new InvariantError(23);
    }
};

var createSignalIfSupported = function () {
    if (typeof AbortController === 'undefined')
        return { controller: false, signal: false };
    var controller = new AbortController();
    var signal = controller.signal;
    return { controller: controller, signal: signal };
};

var selectURI = function (operation, fallbackURI) {
    var context = operation.getContext();
    var contextURI = context.uri;
    if (contextURI) {
        return contextURI;
    }
    else if (typeof fallbackURI === 'function') {
        return fallbackURI(operation);
    }
    else {
        return fallbackURI || '/graphql';
    }
};

function rewriteURIForGET(chosenURI, body) {
    var queryParams = [];
    var addQueryParam = function (key, value) {
        queryParams.push("".concat(key, "=").concat(encodeURIComponent(value)));
    };
    if ('query' in body) {
        addQueryParam('query', body.query);
    }
    if (body.operationName) {
        addQueryParam('operationName', body.operationName);
    }
    if (body.variables) {
        var serializedVariables = void 0;
        try {
            serializedVariables = serializeFetchParameter(body.variables, 'Variables map');
        }
        catch (parseError) {
            return { parseError: parseError };
        }
        addQueryParam('variables', serializedVariables);
    }
    if (body.extensions) {
        var serializedExtensions = void 0;
        try {
            serializedExtensions = serializeFetchParameter(body.extensions, 'Extensions map');
        }
        catch (parseError) {
            return { parseError: parseError };
        }
        addQueryParam('extensions', serializedExtensions);
    }
    var fragment = '', preFragment = chosenURI;
    var fragmentStart = chosenURI.indexOf('#');
    if (fragmentStart !== -1) {
        fragment = chosenURI.substr(fragmentStart);
        preFragment = chosenURI.substr(0, fragmentStart);
    }
    var queryParamsPrefix = preFragment.indexOf('?') === -1 ? '?' : '&';
    var newURI = preFragment + queryParamsPrefix + queryParams.join('&') + fragment;
    return { newURI: newURI };
}

var backupFetch = maybe$2(function () { return fetch; });
var createHttpLink = function (linkOptions) {
    if (linkOptions === void 0) { linkOptions = {}; }
    var _a = linkOptions.uri, uri = _a === void 0 ? '/graphql' : _a, preferredFetch = linkOptions.fetch, _b = linkOptions.print, print = _b === void 0 ? defaultPrinter : _b, includeExtensions = linkOptions.includeExtensions, preserveHeaderCase = linkOptions.preserveHeaderCase, useGETForQueries = linkOptions.useGETForQueries, _c = linkOptions.includeUnusedVariables, includeUnusedVariables = _c === void 0 ? false : _c, requestOptions = __rest(linkOptions, ["uri", "fetch", "print", "includeExtensions", "preserveHeaderCase", "useGETForQueries", "includeUnusedVariables"]);
    if (__DEV__) {
        checkFetcher(preferredFetch || backupFetch);
    }
    var linkConfig = {
        http: { includeExtensions: includeExtensions, preserveHeaderCase: preserveHeaderCase },
        options: requestOptions.fetchOptions,
        credentials: requestOptions.credentials,
        headers: requestOptions.headers,
    };
    return new ApolloLink(function (operation) {
        var chosenURI = selectURI(operation, uri);
        var context = operation.getContext();
        var clientAwarenessHeaders = {};
        if (context.clientAwareness) {
            var _a = context.clientAwareness, name_1 = _a.name, version = _a.version;
            if (name_1) {
                clientAwarenessHeaders['apollographql-client-name'] = name_1;
            }
            if (version) {
                clientAwarenessHeaders['apollographql-client-version'] = version;
            }
        }
        var contextHeaders = __assign$2(__assign$2({}, clientAwarenessHeaders), context.headers);
        var contextConfig = {
            http: context.http,
            options: context.fetchOptions,
            credentials: context.credentials,
            headers: contextHeaders,
        };
        var _b = selectHttpOptionsAndBodyInternal(operation, print, fallbackHttpConfig, linkConfig, contextConfig), options = _b.options, body = _b.body;
        if (body.variables && !includeUnusedVariables) {
            var unusedNames_1 = new Set(Object.keys(body.variables));
            visit(operation.query, {
                Variable: function (node, _key, parent) {
                    if (parent && parent.kind !== 'VariableDefinition') {
                        unusedNames_1.delete(node.name.value);
                    }
                },
            });
            if (unusedNames_1.size) {
                body.variables = __assign$2({}, body.variables);
                unusedNames_1.forEach(function (name) {
                    delete body.variables[name];
                });
            }
        }
        var controller;
        if (!options.signal) {
            var _c = createSignalIfSupported(), _controller = _c.controller, signal = _c.signal;
            controller = _controller;
            if (controller)
                options.signal = signal;
        }
        var definitionIsMutation = function (d) {
            return d.kind === 'OperationDefinition' && d.operation === 'mutation';
        };
        if (useGETForQueries &&
            !operation.query.definitions.some(definitionIsMutation)) {
            options.method = 'GET';
        }
        if (hasDirectives(['defer'], operation.query)) {
            options.headers = options.headers || {};
            options.headers.accept = "multipart/mixed; deferSpec=20220824, application/json";
        }
        if (options.method === 'GET') {
            var _d = rewriteURIForGET(chosenURI, body), newURI = _d.newURI, parseError = _d.parseError;
            if (parseError) {
                return fromError(parseError);
            }
            chosenURI = newURI;
        }
        else {
            try {
                options.body = serializeFetchParameter(body, 'Payload');
            }
            catch (parseError) {
                return fromError(parseError);
            }
        }
        return new Observable(function (observer) {
            var currentFetch = preferredFetch || maybe$2(function () { return fetch; }) || backupFetch;
            currentFetch(chosenURI, options)
                .then(function (response) {
                var _a;
                operation.setContext({ response: response });
                var ctype = (_a = response.headers) === null || _a === void 0 ? void 0 : _a.get('content-type');
                if (ctype !== null && /^multipart\/mixed/i.test(ctype)) {
                    return readMultipartBody(response, observer);
                }
                else {
                    return readJsonBody(response, operation, observer);
                }
            })
                .catch(function (err) { return handleError(err, observer); });
            return function () {
                if (controller)
                    controller.abort();
            };
        });
    });
};

var HttpLink = (function (_super) {
    __extends(HttpLink, _super);
    function HttpLink(options) {
        if (options === void 0) { options = {}; }
        var _this = _super.call(this, createHttpLink(options).request) || this;
        _this.options = options;
        return _this;
    }
    return HttpLink;
}(ApolloLink));

const { toString, hasOwnProperty: hasOwnProperty$3 } = Object.prototype;
const fnToStr = Function.prototype.toString;
const previousComparisons = new Map();
/**
 * Performs a deep equality check on two JavaScript values, tolerating cycles.
 */
function equal(a, b) {
    try {
        return check(a, b);
    }
    finally {
        previousComparisons.clear();
    }
}
function check(a, b) {
    // If the two values are strictly equal, our job is easy.
    if (a === b) {
        return true;
    }
    // Object.prototype.toString returns a representation of the runtime type of
    // the given value that is considerably more precise than typeof.
    const aTag = toString.call(a);
    const bTag = toString.call(b);
    // If the runtime types of a and b are different, they could maybe be equal
    // under some interpretation of equality, but for simplicity and performance
    // we just return false instead.
    if (aTag !== bTag) {
        return false;
    }
    switch (aTag) {
        case '[object Array]':
            // Arrays are a lot like other objects, but we can cheaply compare their
            // lengths as a short-cut before comparing their elements.
            if (a.length !== b.length)
                return false;
        // Fall through to object case...
        case '[object Object]': {
            if (previouslyCompared(a, b))
                return true;
            const aKeys = definedKeys(a);
            const bKeys = definedKeys(b);
            // If `a` and `b` have a different number of enumerable keys, they
            // must be different.
            const keyCount = aKeys.length;
            if (keyCount !== bKeys.length)
                return false;
            // Now make sure they have the same keys.
            for (let k = 0; k < keyCount; ++k) {
                if (!hasOwnProperty$3.call(b, aKeys[k])) {
                    return false;
                }
            }
            // Finally, check deep equality of all child properties.
            for (let k = 0; k < keyCount; ++k) {
                const key = aKeys[k];
                if (!check(a[key], b[key])) {
                    return false;
                }
            }
            return true;
        }
        case '[object Error]':
            return a.name === b.name && a.message === b.message;
        case '[object Number]':
            // Handle NaN, which is !== itself.
            if (a !== a)
                return b !== b;
        // Fall through to shared +a === +b case...
        case '[object Boolean]':
        case '[object Date]':
            return +a === +b;
        case '[object RegExp]':
        case '[object String]':
            return a == `${b}`;
        case '[object Map]':
        case '[object Set]': {
            if (a.size !== b.size)
                return false;
            if (previouslyCompared(a, b))
                return true;
            const aIterator = a.entries();
            const isMap = aTag === '[object Map]';
            while (true) {
                const info = aIterator.next();
                if (info.done)
                    break;
                // If a instanceof Set, aValue === aKey.
                const [aKey, aValue] = info.value;
                // So this works the same way for both Set and Map.
                if (!b.has(aKey)) {
                    return false;
                }
                // However, we care about deep equality of values only when dealing
                // with Map structures.
                if (isMap && !check(aValue, b.get(aKey))) {
                    return false;
                }
            }
            return true;
        }
        case '[object Uint16Array]':
        case '[object Uint8Array]': // Buffer, in Node.js.
        case '[object Uint32Array]':
        case '[object Int32Array]':
        case '[object Int8Array]':
        case '[object Int16Array]':
        case '[object ArrayBuffer]':
            // DataView doesn't need these conversions, but the equality check is
            // otherwise the same.
            a = new Uint8Array(a);
            b = new Uint8Array(b);
        // Fall through...
        case '[object DataView]': {
            let len = a.byteLength;
            if (len === b.byteLength) {
                while (len-- && a[len] === b[len]) {
                    // Keep looping as long as the bytes are equal.
                }
            }
            return len === -1;
        }
        case '[object AsyncFunction]':
        case '[object GeneratorFunction]':
        case '[object AsyncGeneratorFunction]':
        case '[object Function]': {
            const aCode = fnToStr.call(a);
            if (aCode !== fnToStr.call(b)) {
                return false;
            }
            // We consider non-native functions equal if they have the same code
            // (native functions require === because their code is censored).
            // Note that this behavior is not entirely sound, since !== function
            // objects with the same code can behave differently depending on
            // their closure scope. However, any function can behave differently
            // depending on the values of its input arguments (including this)
            // and its calling context (including its closure scope), even
            // though the function object is === to itself; and it is entirely
            // possible for functions that are not === to behave exactly the
            // same under all conceivable circumstances. Because none of these
            // factors are statically decidable in JavaScript, JS function
            // equality is not well-defined. This ambiguity allows us to
            // consider the best possible heuristic among various imperfect
            // options, and equating non-native functions that have the same
            // code has enormous practical benefits, such as when comparing
            // functions that are repeatedly passed as fresh function
            // expressions within objects that are otherwise deeply equal. Since
            // any function created from the same syntactic expression (in the
            // same code location) will always stringify to the same code
            // according to fnToStr.call, we can reasonably expect these
            // repeatedly passed function expressions to have the same code, and
            // thus behave "the same" (with all the caveats mentioned above),
            // even though the runtime function objects are !== to one another.
            return !endsWith(aCode, nativeCodeSuffix);
        }
    }
    // Otherwise the values are not equal.
    return false;
}
function definedKeys(obj) {
    // Remember that the second argument to Array.prototype.filter will be
    // used as `this` within the callback function.
    return Object.keys(obj).filter(isDefinedKey, obj);
}
function isDefinedKey(key) {
    return this[key] !== void 0;
}
const nativeCodeSuffix = "{ [native code] }";
function endsWith(full, suffix) {
    const fromIndex = full.length - suffix.length;
    return fromIndex >= 0 &&
        full.indexOf(suffix, fromIndex) === fromIndex;
}
function previouslyCompared(a, b) {
    // Though cyclic references can make an object graph appear infinite from the
    // perspective of a depth-first traversal, the graph still contains a finite
    // number of distinct object references. We use the previousComparisons cache
    // to avoid comparing the same pair of object references more than once, which
    // guarantees termination (even if we end up comparing every object in one
    // graph to every object in the other graph, which is extremely unlikely),
    // while still allowing weird isomorphic structures (like rings with different
    // lengths) a chance to pass the equality test.
    let bSet = previousComparisons.get(a);
    if (bSet) {
        // Return true here because we can be sure false will be returned somewhere
        // else if the objects are not equivalent.
        if (bSet.has(b))
            return true;
    }
    else {
        previousComparisons.set(a, bSet = new Set);
    }
    bSet.add(b);
    return false;
}

// A [trie](https://en.wikipedia.org/wiki/Trie) data structure that holds
// object keys weakly, yet can also hold non-object keys, unlike the
// native `WeakMap`.
// If no makeData function is supplied, the looked-up data will be an empty,
// null-prototype Object.
var defaultMakeData = function () { return Object.create(null); };
// Useful for processing arguments objects as well as arrays.
var _a$1 = Array.prototype, forEach = _a$1.forEach, slice = _a$1.slice;
var Trie = /** @class */ (function () {
    function Trie(weakness, makeData) {
        if (weakness === void 0) { weakness = true; }
        if (makeData === void 0) { makeData = defaultMakeData; }
        this.weakness = weakness;
        this.makeData = makeData;
    }
    Trie.prototype.lookup = function () {
        var array = [];
        for (var _i = 0; _i < arguments.length; _i++) {
            array[_i] = arguments[_i];
        }
        return this.lookupArray(array);
    };
    Trie.prototype.lookupArray = function (array) {
        var node = this;
        forEach.call(array, function (key) { return node = node.getChildTrie(key); });
        return node.data || (node.data = this.makeData(slice.call(array)));
    };
    Trie.prototype.getChildTrie = function (key) {
        var map = this.weakness && isObjRef(key)
            ? this.weak || (this.weak = new WeakMap())
            : this.strong || (this.strong = new Map());
        var child = map.get(key);
        if (!child)
            map.set(key, child = new Trie(this.weakness, this.makeData));
        return child;
    };
    return Trie;
}());
function isObjRef(value) {
    switch (typeof value) {
        case "object":
            if (value === null)
                break;
        // Fall through to return true...
        case "function":
            return true;
    }
    return false;
}

// This currentContext variable will only be used if the makeSlotClass
// function is called, which happens only if this is the first copy of the
// @wry/context package to be imported.
let currentContext = null;
// This unique internal object is used to denote the absence of a value
// for a given Slot, and is never exposed to outside code.
const MISSING_VALUE = {};
let idCounter = 1;
// Although we can't do anything about the cost of duplicated code from
// accidentally bundling multiple copies of the @wry/context package, we can
// avoid creating the Slot class more than once using makeSlotClass.
const makeSlotClass = () => class Slot {
    constructor() {
        // If you have a Slot object, you can find out its slot.id, but you cannot
        // guess the slot.id of a Slot you don't have access to, thanks to the
        // randomized suffix.
        this.id = [
            "slot",
            idCounter++,
            Date.now(),
            Math.random().toString(36).slice(2),
        ].join(":");
    }
    hasValue() {
        for (let context = currentContext; context; context = context.parent) {
            // We use the Slot object iself as a key to its value, which means the
            // value cannot be obtained without a reference to the Slot object.
            if (this.id in context.slots) {
                const value = context.slots[this.id];
                if (value === MISSING_VALUE)
                    break;
                if (context !== currentContext) {
                    // Cache the value in currentContext.slots so the next lookup will
                    // be faster. This caching is safe because the tree of contexts and
                    // the values of the slots are logically immutable.
                    currentContext.slots[this.id] = value;
                }
                return true;
            }
        }
        if (currentContext) {
            // If a value was not found for this Slot, it's never going to be found
            // no matter how many times we look it up, so we might as well cache
            // the absence of the value, too.
            currentContext.slots[this.id] = MISSING_VALUE;
        }
        return false;
    }
    getValue() {
        if (this.hasValue()) {
            return currentContext.slots[this.id];
        }
    }
    withValue(value, callback, 
    // Given the prevalence of arrow functions, specifying arguments is likely
    // to be much more common than specifying `this`, hence this ordering:
    args, thisArg) {
        const slots = {
            __proto__: null,
            [this.id]: value,
        };
        const parent = currentContext;
        currentContext = { parent, slots };
        try {
            // Function.prototype.apply allows the arguments array argument to be
            // omitted or undefined, so args! is fine here.
            return callback.apply(thisArg, args);
        }
        finally {
            currentContext = parent;
        }
    }
    // Capture the current context and wrap a callback function so that it
    // reestablishes the captured context when called.
    static bind(callback) {
        const context = currentContext;
        return function () {
            const saved = currentContext;
            try {
                currentContext = context;
                return callback.apply(this, arguments);
            }
            finally {
                currentContext = saved;
            }
        };
    }
    // Immediately run a callback function without any captured context.
    static noContext(callback, 
    // Given the prevalence of arrow functions, specifying arguments is likely
    // to be much more common than specifying `this`, hence this ordering:
    args, thisArg) {
        if (currentContext) {
            const saved = currentContext;
            try {
                currentContext = null;
                // Function.prototype.apply allows the arguments array argument to be
                // omitted or undefined, so args! is fine here.
                return callback.apply(thisArg, args);
            }
            finally {
                currentContext = saved;
            }
        }
        else {
            return callback.apply(thisArg, args);
        }
    }
};
function maybe(fn) {
    try {
        return fn();
    }
    catch (ignored) { }
}
// We store a single global implementation of the Slot class as a permanent
// non-enumerable property of the globalThis object. This obfuscation does
// nothing to prevent access to the Slot class, but at least it ensures the
// implementation (i.e. currentContext) cannot be tampered with, and all copies
// of the @wry/context package (hopefully just one) will share the same Slot
// implementation. Since the first copy of the @wry/context package to be
// imported wins, this technique imposes a steep cost for any future breaking
// changes to the Slot class.
const globalKey = "@wry/context:Slot";
const host = 
// Prefer globalThis when available.
// https://github.com/benjamn/wryware/issues/347
maybe(() => globalThis) ||
    // Fall back to global, which works in Node.js and may be converted by some
    // bundlers to the appropriate identifier (window, self, ...) depending on the
    // bundling target. https://github.com/endojs/endo/issues/576#issuecomment-1178515224
    maybe(() => global) ||
    // Otherwise, use a dummy host that's local to this module. We used to fall
    // back to using the Array constructor as a namespace, but that was flagged in
    // https://github.com/benjamn/wryware/issues/347, and can be avoided.
    Object.create(null);
// Whichever globalHost we're using, make TypeScript happy about the additional
// globalKey property.
const globalHost = host;
const Slot = globalHost[globalKey] ||
    // Earlier versions of this package stored the globalKey property on the Array
    // constructor, so we check there as well, to prevent Slot class duplication.
    Array[globalKey] ||
    (function (Slot) {
        try {
            Object.defineProperty(globalHost, globalKey, {
                value: Slot,
                enumerable: false,
                writable: false,
                // When it was possible for globalHost to be the Array constructor (a
                // legacy Slot dedup strategy), it was important for the property to be
                // configurable:true so it could be deleted. That does not seem to be as
                // important when globalHost is the global object, but I don't want to
                // cause similar problems again, and configurable:true seems safest.
                // https://github.com/endojs/endo/issues/576#issuecomment-1178274008
                configurable: true
            });
        }
        finally {
            return Slot;
        }
    })(makeSlotClass());

function defaultDispose() { }
var Cache = /** @class */ (function () {
    function Cache(max, dispose) {
        if (max === void 0) { max = Infinity; }
        if (dispose === void 0) { dispose = defaultDispose; }
        this.max = max;
        this.dispose = dispose;
        this.map = new Map();
        this.newest = null;
        this.oldest = null;
    }
    Cache.prototype.has = function (key) {
        return this.map.has(key);
    };
    Cache.prototype.get = function (key) {
        var node = this.getNode(key);
        return node && node.value;
    };
    Cache.prototype.getNode = function (key) {
        var node = this.map.get(key);
        if (node && node !== this.newest) {
            var older = node.older, newer = node.newer;
            if (newer) {
                newer.older = older;
            }
            if (older) {
                older.newer = newer;
            }
            node.older = this.newest;
            node.older.newer = node;
            node.newer = null;
            this.newest = node;
            if (node === this.oldest) {
                this.oldest = newer;
            }
        }
        return node;
    };
    Cache.prototype.set = function (key, value) {
        var node = this.getNode(key);
        if (node) {
            return node.value = value;
        }
        node = {
            key: key,
            value: value,
            newer: null,
            older: this.newest
        };
        if (this.newest) {
            this.newest.newer = node;
        }
        this.newest = node;
        this.oldest = this.oldest || node;
        this.map.set(key, node);
        return node.value;
    };
    Cache.prototype.clean = function () {
        while (this.oldest && this.map.size > this.max) {
            this.delete(this.oldest.key);
        }
    };
    Cache.prototype.delete = function (key) {
        var node = this.map.get(key);
        if (node) {
            if (node === this.newest) {
                this.newest = node.older;
            }
            if (node === this.oldest) {
                this.oldest = node.newer;
            }
            if (node.newer) {
                node.newer.older = node.older;
            }
            if (node.older) {
                node.older.newer = node.newer;
            }
            this.map.delete(key);
            this.dispose(node.value, key);
            return true;
        }
        return false;
    };
    return Cache;
}());

var parentEntrySlot = new Slot();

var _a;
var hasOwnProperty$2 = Object.prototype.hasOwnProperty;
var 
// This Array.from polyfill is restricted to working with Set<any> for now,
// but we can improve the polyfill and add other input types, as needed. Note
// that this fallback implementation will only be used if the host environment
// does not support a native Array.from function. In most modern JS runtimes,
// the toArray function exported here will be === Array.from.
toArray = (_a = Array.from, _a === void 0 ? function (collection) {
    var array = [];
    collection.forEach(function (item) { return array.push(item); });
    return array;
} : _a);
function maybeUnsubscribe(entryOrDep) {
    var unsubscribe = entryOrDep.unsubscribe;
    if (typeof unsubscribe === "function") {
        entryOrDep.unsubscribe = void 0;
        unsubscribe();
    }
}

var emptySetPool = [];
var POOL_TARGET_SIZE = 100;
// Since this package might be used browsers, we should avoid using the
// Node built-in assert module.
function assert(condition, optionalMessage) {
    if (!condition) {
        throw new Error(optionalMessage || "assertion failure");
    }
}
function valueIs(a, b) {
    var len = a.length;
    return (
    // Unknown values are not equal to each other.
    len > 0 &&
        // Both values must be ordinary (or both exceptional) to be equal.
        len === b.length &&
        // The underlying value or exception must be the same.
        a[len - 1] === b[len - 1]);
}
function valueGet(value) {
    switch (value.length) {
        case 0: throw new Error("unknown value");
        case 1: return value[0];
        case 2: throw value[1];
    }
}
function valueCopy(value) {
    return value.slice(0);
}
var Entry = /** @class */ (function () {
    function Entry(fn) {
        this.fn = fn;
        this.parents = new Set();
        this.childValues = new Map();
        // When this Entry has children that are dirty, this property becomes
        // a Set containing other Entry objects, borrowed from emptySetPool.
        // When the set becomes empty, it gets recycled back to emptySetPool.
        this.dirtyChildren = null;
        this.dirty = true;
        this.recomputing = false;
        this.value = [];
        this.deps = null;
        ++Entry.count;
    }
    Entry.prototype.peek = function () {
        if (this.value.length === 1 && !mightBeDirty(this)) {
            rememberParent(this);
            return this.value[0];
        }
    };
    // This is the most important method of the Entry API, because it
    // determines whether the cached this.value can be returned immediately,
    // or must be recomputed. The overall performance of the caching system
    // depends on the truth of the following observations: (1) this.dirty is
    // usually false, (2) this.dirtyChildren is usually null/empty, and thus
    // (3) valueGet(this.value) is usually returned without recomputation.
    Entry.prototype.recompute = function (args) {
        assert(!this.recomputing, "already recomputing");
        rememberParent(this);
        return mightBeDirty(this)
            ? reallyRecompute(this, args)
            : valueGet(this.value);
    };
    Entry.prototype.setDirty = function () {
        if (this.dirty)
            return;
        this.dirty = true;
        this.value.length = 0;
        reportDirty(this);
        // We can go ahead and unsubscribe here, since any further dirty
        // notifications we receive will be redundant, and unsubscribing may
        // free up some resources, e.g. file watchers.
        maybeUnsubscribe(this);
    };
    Entry.prototype.dispose = function () {
        var _this = this;
        this.setDirty();
        // Sever any dependency relationships with our own children, so those
        // children don't retain this parent Entry in their child.parents sets,
        // thereby preventing it from being fully garbage collected.
        forgetChildren(this);
        // Because this entry has been kicked out of the cache (in index.js),
        // we've lost the ability to find out if/when this entry becomes dirty,
        // whether that happens through a subscription, because of a direct call
        // to entry.setDirty(), or because one of its children becomes dirty.
        // Because of this loss of future information, we have to assume the
        // worst (that this entry might have become dirty very soon), so we must
        // immediately mark this entry's parents as dirty. Normally we could
        // just call entry.setDirty() rather than calling parent.setDirty() for
        // each parent, but that would leave this entry in parent.childValues
        // and parent.dirtyChildren, which would prevent the child from being
        // truly forgotten.
        eachParent(this, function (parent, child) {
            parent.setDirty();
            forgetChild(parent, _this);
        });
    };
    Entry.prototype.forget = function () {
        // The code that creates Entry objects in index.ts will replace this method
        // with one that actually removes the Entry from the cache, which will also
        // trigger the entry.dispose method.
        this.dispose();
    };
    Entry.prototype.dependOn = function (dep) {
        dep.add(this);
        if (!this.deps) {
            this.deps = emptySetPool.pop() || new Set();
        }
        this.deps.add(dep);
    };
    Entry.prototype.forgetDeps = function () {
        var _this = this;
        if (this.deps) {
            toArray(this.deps).forEach(function (dep) { return dep.delete(_this); });
            this.deps.clear();
            emptySetPool.push(this.deps);
            this.deps = null;
        }
    };
    Entry.count = 0;
    return Entry;
}());
function rememberParent(child) {
    var parent = parentEntrySlot.getValue();
    if (parent) {
        child.parents.add(parent);
        if (!parent.childValues.has(child)) {
            parent.childValues.set(child, []);
        }
        if (mightBeDirty(child)) {
            reportDirtyChild(parent, child);
        }
        else {
            reportCleanChild(parent, child);
        }
        return parent;
    }
}
function reallyRecompute(entry, args) {
    forgetChildren(entry);
    // Set entry as the parent entry while calling recomputeNewValue(entry).
    parentEntrySlot.withValue(entry, recomputeNewValue, [entry, args]);
    if (maybeSubscribe(entry, args)) {
        // If we successfully recomputed entry.value and did not fail to
        // (re)subscribe, then this Entry is no longer explicitly dirty.
        setClean(entry);
    }
    return valueGet(entry.value);
}
function recomputeNewValue(entry, args) {
    entry.recomputing = true;
    // Set entry.value as unknown.
    entry.value.length = 0;
    try {
        // If entry.fn succeeds, entry.value will become a normal Value.
        entry.value[0] = entry.fn.apply(null, args);
    }
    catch (e) {
        // If entry.fn throws, entry.value will become exceptional.
        entry.value[1] = e;
    }
    // Either way, this line is always reached.
    entry.recomputing = false;
}
function mightBeDirty(entry) {
    return entry.dirty || !!(entry.dirtyChildren && entry.dirtyChildren.size);
}
function setClean(entry) {
    entry.dirty = false;
    if (mightBeDirty(entry)) {
        // This Entry may still have dirty children, in which case we can't
        // let our parents know we're clean just yet.
        return;
    }
    reportClean(entry);
}
function reportDirty(child) {
    eachParent(child, reportDirtyChild);
}
function reportClean(child) {
    eachParent(child, reportCleanChild);
}
function eachParent(child, callback) {
    var parentCount = child.parents.size;
    if (parentCount) {
        var parents = toArray(child.parents);
        for (var i = 0; i < parentCount; ++i) {
            callback(parents[i], child);
        }
    }
}
// Let a parent Entry know that one of its children may be dirty.
function reportDirtyChild(parent, child) {
    // Must have called rememberParent(child) before calling
    // reportDirtyChild(parent, child).
    assert(parent.childValues.has(child));
    assert(mightBeDirty(child));
    var parentWasClean = !mightBeDirty(parent);
    if (!parent.dirtyChildren) {
        parent.dirtyChildren = emptySetPool.pop() || new Set;
    }
    else if (parent.dirtyChildren.has(child)) {
        // If we already know this child is dirty, then we must have already
        // informed our own parents that we are dirty, so we can terminate
        // the recursion early.
        return;
    }
    parent.dirtyChildren.add(child);
    // If parent was clean before, it just became (possibly) dirty (according to
    // mightBeDirty), since we just added child to parent.dirtyChildren.
    if (parentWasClean) {
        reportDirty(parent);
    }
}
// Let a parent Entry know that one of its children is no longer dirty.
function reportCleanChild(parent, child) {
    // Must have called rememberChild(child) before calling
    // reportCleanChild(parent, child).
    assert(parent.childValues.has(child));
    assert(!mightBeDirty(child));
    var childValue = parent.childValues.get(child);
    if (childValue.length === 0) {
        parent.childValues.set(child, valueCopy(child.value));
    }
    else if (!valueIs(childValue, child.value)) {
        parent.setDirty();
    }
    removeDirtyChild(parent, child);
    if (mightBeDirty(parent)) {
        return;
    }
    reportClean(parent);
}
function removeDirtyChild(parent, child) {
    var dc = parent.dirtyChildren;
    if (dc) {
        dc.delete(child);
        if (dc.size === 0) {
            if (emptySetPool.length < POOL_TARGET_SIZE) {
                emptySetPool.push(dc);
            }
            parent.dirtyChildren = null;
        }
    }
}
// Removes all children from this entry and returns an array of the
// removed children.
function forgetChildren(parent) {
    if (parent.childValues.size > 0) {
        parent.childValues.forEach(function (_value, child) {
            forgetChild(parent, child);
        });
    }
    // Remove this parent Entry from any sets to which it was added by the
    // addToSet method.
    parent.forgetDeps();
    // After we forget all our children, this.dirtyChildren must be empty
    // and therefore must have been reset to null.
    assert(parent.dirtyChildren === null);
}
function forgetChild(parent, child) {
    child.parents.delete(parent);
    parent.childValues.delete(child);
    removeDirtyChild(parent, child);
}
function maybeSubscribe(entry, args) {
    if (typeof entry.subscribe === "function") {
        try {
            maybeUnsubscribe(entry); // Prevent double subscriptions.
            entry.unsubscribe = entry.subscribe.apply(null, args);
        }
        catch (e) {
            // If this Entry has a subscribe function and it threw an exception
            // (or an unsubscribe function it previously returned now throws),
            // return false to indicate that we were not able to subscribe (or
            // unsubscribe), and this Entry should remain dirty.
            entry.setDirty();
            return false;
        }
    }
    // Returning true indicates either that there was no entry.subscribe
    // function or that it succeeded.
    return true;
}

var EntryMethods = {
    setDirty: true,
    dispose: true,
    forget: true, // Fully remove parent Entry from LRU cache and computation graph
};
function dep(options) {
    var depsByKey = new Map();
    var subscribe = options && options.subscribe;
    function depend(key) {
        var parent = parentEntrySlot.getValue();
        if (parent) {
            var dep_1 = depsByKey.get(key);
            if (!dep_1) {
                depsByKey.set(key, dep_1 = new Set);
            }
            parent.dependOn(dep_1);
            if (typeof subscribe === "function") {
                maybeUnsubscribe(dep_1);
                dep_1.unsubscribe = subscribe(key);
            }
        }
    }
    depend.dirty = function dirty(key, entryMethodName) {
        var dep = depsByKey.get(key);
        if (dep) {
            var m_1 = (entryMethodName &&
                hasOwnProperty$2.call(EntryMethods, entryMethodName)) ? entryMethodName : "setDirty";
            // We have to use toArray(dep).forEach instead of dep.forEach, because
            // modifying a Set while iterating over it can cause elements in the Set
            // to be removed from the Set before they've been iterated over.
            toArray(dep).forEach(function (entry) { return entry[m_1](); });
            depsByKey.delete(key);
            maybeUnsubscribe(dep);
        }
    };
    return depend;
}

function makeDefaultMakeCacheKeyFunction() {
    var keyTrie = new Trie(typeof WeakMap === "function");
    return function () {
        return keyTrie.lookupArray(arguments);
    };
}
// The defaultMakeCacheKey function is remarkably powerful, because it gives
// a unique object for any shallow-identical list of arguments. If you need
// to implement a custom makeCacheKey function, you may find it helpful to
// delegate the final work to defaultMakeCacheKey, which is why we export it
// here. However, you may want to avoid defaultMakeCacheKey if your runtime
// does not support WeakMap, or you have the ability to return a string key.
// In those cases, just write your own custom makeCacheKey functions.
makeDefaultMakeCacheKeyFunction();
var caches = new Set();
function wrap(originalFunction, options) {
    if (options === void 0) { options = Object.create(null); }
    var cache = new Cache(options.max || Math.pow(2, 16), function (entry) { return entry.dispose(); });
    var keyArgs = options.keyArgs;
    var makeCacheKey = options.makeCacheKey ||
        makeDefaultMakeCacheKeyFunction();
    var optimistic = function () {
        var key = makeCacheKey.apply(null, keyArgs ? keyArgs.apply(null, arguments) : arguments);
        if (key === void 0) {
            return originalFunction.apply(null, arguments);
        }
        var entry = cache.get(key);
        if (!entry) {
            cache.set(key, entry = new Entry(originalFunction));
            entry.subscribe = options.subscribe;
            // Give the Entry the ability to trigger cache.delete(key), even though
            // the Entry itself does not know about key or cache.
            entry.forget = function () { return cache.delete(key); };
        }
        var value = entry.recompute(Array.prototype.slice.call(arguments));
        // Move this entry to the front of the least-recently used queue,
        // since we just finished computing its value.
        cache.set(key, entry);
        caches.add(cache);
        // Clean up any excess entries in the cache, but only if there is no
        // active parent entry, meaning we're not in the middle of a larger
        // computation that might be flummoxed by the cleaning.
        if (!parentEntrySlot.hasValue()) {
            caches.forEach(function (cache) { return cache.clean(); });
            caches.clear();
        }
        return value;
    };
    Object.defineProperty(optimistic, "size", {
        get: function () {
            return cache["map"].size;
        },
        configurable: false,
        enumerable: false,
    });
    function dirtyKey(key) {
        var entry = cache.get(key);
        if (entry) {
            entry.setDirty();
        }
    }
    optimistic.dirtyKey = dirtyKey;
    optimistic.dirty = function dirty() {
        dirtyKey(makeCacheKey.apply(null, arguments));
    };
    function peekKey(key) {
        var entry = cache.get(key);
        if (entry) {
            return entry.peek();
        }
    }
    optimistic.peekKey = peekKey;
    optimistic.peek = function peek() {
        return peekKey(makeCacheKey.apply(null, arguments));
    };
    function forgetKey(key) {
        return cache.delete(key);
    }
    optimistic.forgetKey = forgetKey;
    optimistic.forget = function forget() {
        return forgetKey(makeCacheKey.apply(null, arguments));
    };
    optimistic.makeCacheKey = makeCacheKey;
    optimistic.getKey = keyArgs ? function getKey() {
        return makeCacheKey.apply(null, keyArgs.apply(null, arguments));
    } : makeCacheKey;
    return Object.freeze(optimistic);
}

var ApolloCache = (function () {
    function ApolloCache() {
        this.getFragmentDoc = wrap(getFragmentQueryDocument);
    }
    ApolloCache.prototype.batch = function (options) {
        var _this = this;
        var optimisticId = typeof options.optimistic === "string" ? options.optimistic :
            options.optimistic === false ? null : void 0;
        var updateResult;
        this.performTransaction(function () { return updateResult = options.update(_this); }, optimisticId);
        return updateResult;
    };
    ApolloCache.prototype.recordOptimisticTransaction = function (transaction, optimisticId) {
        this.performTransaction(transaction, optimisticId);
    };
    ApolloCache.prototype.transformDocument = function (document) {
        return document;
    };
    ApolloCache.prototype.transformForLink = function (document) {
        return document;
    };
    ApolloCache.prototype.identify = function (object) {
        return;
    };
    ApolloCache.prototype.gc = function () {
        return [];
    };
    ApolloCache.prototype.modify = function (options) {
        return false;
    };
    ApolloCache.prototype.readQuery = function (options, optimistic) {
        if (optimistic === void 0) { optimistic = !!options.optimistic; }
        return this.read(__assign$2(__assign$2({}, options), { rootId: options.id || 'ROOT_QUERY', optimistic: optimistic }));
    };
    ApolloCache.prototype.readFragment = function (options, optimistic) {
        if (optimistic === void 0) { optimistic = !!options.optimistic; }
        return this.read(__assign$2(__assign$2({}, options), { query: this.getFragmentDoc(options.fragment, options.fragmentName), rootId: options.id, optimistic: optimistic }));
    };
    ApolloCache.prototype.writeQuery = function (_a) {
        var id = _a.id, data = _a.data, options = __rest(_a, ["id", "data"]);
        return this.write(Object.assign(options, {
            dataId: id || 'ROOT_QUERY',
            result: data,
        }));
    };
    ApolloCache.prototype.writeFragment = function (_a) {
        var id = _a.id, data = _a.data, fragment = _a.fragment, fragmentName = _a.fragmentName, options = __rest(_a, ["id", "data", "fragment", "fragmentName"]);
        return this.write(Object.assign(options, {
            query: this.getFragmentDoc(fragment, fragmentName),
            dataId: id,
            result: data,
        }));
    };
    ApolloCache.prototype.updateQuery = function (options, update) {
        return this.batch({
            update: function (cache) {
                var value = cache.readQuery(options);
                var data = update(value);
                if (data === void 0 || data === null)
                    return value;
                cache.writeQuery(__assign$2(__assign$2({}, options), { data: data }));
                return data;
            },
        });
    };
    ApolloCache.prototype.updateFragment = function (options, update) {
        return this.batch({
            update: function (cache) {
                var value = cache.readFragment(options);
                var data = update(value);
                if (data === void 0 || data === null)
                    return value;
                cache.writeFragment(__assign$2(__assign$2({}, options), { data: data }));
                return data;
            },
        });
    };
    return ApolloCache;
}());

var MissingFieldError = (function (_super) {
    __extends(MissingFieldError, _super);
    function MissingFieldError(message, path, query, variables) {
        var _a;
        var _this = _super.call(this, message) || this;
        _this.message = message;
        _this.path = path;
        _this.query = query;
        _this.variables = variables;
        if (Array.isArray(_this.path)) {
            _this.missing = _this.message;
            for (var i = _this.path.length - 1; i >= 0; --i) {
                _this.missing = (_a = {}, _a[_this.path[i]] = _this.missing, _a);
            }
        }
        else {
            _this.missing = _this.path;
        }
        _this.__proto__ = MissingFieldError.prototype;
        return _this;
    }
    return MissingFieldError;
}(Error));

var hasOwn = Object.prototype.hasOwnProperty;
function isNullish(value) {
    return value === null || value === void 0;
}
function defaultDataIdFromObject(_a, context) {
    var __typename = _a.__typename, id = _a.id, _id = _a._id;
    if (typeof __typename === "string") {
        if (context) {
            context.keyObject =
                !isNullish(id) ? { id: id } :
                    !isNullish(_id) ? { _id: _id } :
                        void 0;
        }
        if (isNullish(id) && !isNullish(_id)) {
            id = _id;
        }
        if (!isNullish(id)) {
            return "".concat(__typename, ":").concat((typeof id === "number" ||
                typeof id === "string") ? id : JSON.stringify(id));
        }
    }
}
var defaultConfig = {
    dataIdFromObject: defaultDataIdFromObject,
    addTypename: true,
    resultCaching: true,
    canonizeResults: false,
};
function normalizeConfig(config) {
    return compact(defaultConfig, config);
}
function shouldCanonizeResults(config) {
    var value = config.canonizeResults;
    return value === void 0 ? defaultConfig.canonizeResults : value;
}
function getTypenameFromStoreObject(store, objectOrReference) {
    return isReference(objectOrReference)
        ? store.get(objectOrReference.__ref, "__typename")
        : objectOrReference && objectOrReference.__typename;
}
var TypeOrFieldNameRegExp = /^[_a-z][_0-9a-z]*/i;
function fieldNameFromStoreName(storeFieldName) {
    var match = storeFieldName.match(TypeOrFieldNameRegExp);
    return match ? match[0] : storeFieldName;
}
function selectionSetMatchesResult(selectionSet, result, variables) {
    if (isNonNullObject(result)) {
        return isArray(result)
            ? result.every(function (item) { return selectionSetMatchesResult(selectionSet, item, variables); })
            : selectionSet.selections.every(function (field) {
                if (isField(field) && shouldInclude(field, variables)) {
                    var key = resultKeyNameFromField(field);
                    return hasOwn.call(result, key) &&
                        (!field.selectionSet ||
                            selectionSetMatchesResult(field.selectionSet, result[key], variables));
                }
                return true;
            });
    }
    return false;
}
function storeValueIsStoreObject(value) {
    return isNonNullObject(value) &&
        !isReference(value) &&
        !isArray(value);
}
function makeProcessedFieldsMerger() {
    return new DeepMerger;
}
function extractFragmentContext(document, fragments) {
    var fragmentMap = createFragmentMap(getFragmentDefinitions(document));
    return {
        fragmentMap: fragmentMap,
        lookupFragment: function (name) {
            var def = fragmentMap[name];
            if (!def && fragments) {
                def = fragments.lookup(name);
            }
            return def || null;
        },
    };
}

var DELETE = Object.create(null);
var delModifier = function () { return DELETE; };
var INVALIDATE = Object.create(null);
var EntityStore = (function () {
    function EntityStore(policies, group) {
        var _this = this;
        this.policies = policies;
        this.group = group;
        this.data = Object.create(null);
        this.rootIds = Object.create(null);
        this.refs = Object.create(null);
        this.getFieldValue = function (objectOrReference, storeFieldName) { return maybeDeepFreeze(isReference(objectOrReference)
            ? _this.get(objectOrReference.__ref, storeFieldName)
            : objectOrReference && objectOrReference[storeFieldName]); };
        this.canRead = function (objOrRef) {
            return isReference(objOrRef)
                ? _this.has(objOrRef.__ref)
                : typeof objOrRef === "object";
        };
        this.toReference = function (objOrIdOrRef, mergeIntoStore) {
            if (typeof objOrIdOrRef === "string") {
                return makeReference(objOrIdOrRef);
            }
            if (isReference(objOrIdOrRef)) {
                return objOrIdOrRef;
            }
            var id = _this.policies.identify(objOrIdOrRef)[0];
            if (id) {
                var ref = makeReference(id);
                if (mergeIntoStore) {
                    _this.merge(id, objOrIdOrRef);
                }
                return ref;
            }
        };
    }
    EntityStore.prototype.toObject = function () {
        return __assign$2({}, this.data);
    };
    EntityStore.prototype.has = function (dataId) {
        return this.lookup(dataId, true) !== void 0;
    };
    EntityStore.prototype.get = function (dataId, fieldName) {
        this.group.depend(dataId, fieldName);
        if (hasOwn.call(this.data, dataId)) {
            var storeObject = this.data[dataId];
            if (storeObject && hasOwn.call(storeObject, fieldName)) {
                return storeObject[fieldName];
            }
        }
        if (fieldName === "__typename" &&
            hasOwn.call(this.policies.rootTypenamesById, dataId)) {
            return this.policies.rootTypenamesById[dataId];
        }
        if (this instanceof Layer) {
            return this.parent.get(dataId, fieldName);
        }
    };
    EntityStore.prototype.lookup = function (dataId, dependOnExistence) {
        if (dependOnExistence)
            this.group.depend(dataId, "__exists");
        if (hasOwn.call(this.data, dataId)) {
            return this.data[dataId];
        }
        if (this instanceof Layer) {
            return this.parent.lookup(dataId, dependOnExistence);
        }
        if (this.policies.rootTypenamesById[dataId]) {
            return Object.create(null);
        }
    };
    EntityStore.prototype.merge = function (older, newer) {
        var _this = this;
        var dataId;
        if (isReference(older))
            older = older.__ref;
        if (isReference(newer))
            newer = newer.__ref;
        var existing = typeof older === "string"
            ? this.lookup(dataId = older)
            : older;
        var incoming = typeof newer === "string"
            ? this.lookup(dataId = newer)
            : newer;
        if (!incoming)
            return;
        __DEV__ ? invariant$2(typeof dataId === "string", "store.merge expects a string ID") : invariant$2(typeof dataId === "string", 1);
        var merged = new DeepMerger(storeObjectReconciler).merge(existing, incoming);
        this.data[dataId] = merged;
        if (merged !== existing) {
            delete this.refs[dataId];
            if (this.group.caching) {
                var fieldsToDirty_1 = Object.create(null);
                if (!existing)
                    fieldsToDirty_1.__exists = 1;
                Object.keys(incoming).forEach(function (storeFieldName) {
                    if (!existing || existing[storeFieldName] !== merged[storeFieldName]) {
                        fieldsToDirty_1[storeFieldName] = 1;
                        var fieldName = fieldNameFromStoreName(storeFieldName);
                        if (fieldName !== storeFieldName &&
                            !_this.policies.hasKeyArgs(merged.__typename, fieldName)) {
                            fieldsToDirty_1[fieldName] = 1;
                        }
                        if (merged[storeFieldName] === void 0 && !(_this instanceof Layer)) {
                            delete merged[storeFieldName];
                        }
                    }
                });
                if (fieldsToDirty_1.__typename &&
                    !(existing && existing.__typename) &&
                    this.policies.rootTypenamesById[dataId] === merged.__typename) {
                    delete fieldsToDirty_1.__typename;
                }
                Object.keys(fieldsToDirty_1).forEach(function (fieldName) { return _this.group.dirty(dataId, fieldName); });
            }
        }
    };
    EntityStore.prototype.modify = function (dataId, fields) {
        var _this = this;
        var storeObject = this.lookup(dataId);
        if (storeObject) {
            var changedFields_1 = Object.create(null);
            var needToMerge_1 = false;
            var allDeleted_1 = true;
            var sharedDetails_1 = {
                DELETE: DELETE,
                INVALIDATE: INVALIDATE,
                isReference: isReference,
                toReference: this.toReference,
                canRead: this.canRead,
                readField: function (fieldNameOrOptions, from) { return _this.policies.readField(typeof fieldNameOrOptions === "string" ? {
                    fieldName: fieldNameOrOptions,
                    from: from || makeReference(dataId),
                } : fieldNameOrOptions, { store: _this }); },
            };
            Object.keys(storeObject).forEach(function (storeFieldName) {
                var fieldName = fieldNameFromStoreName(storeFieldName);
                var fieldValue = storeObject[storeFieldName];
                if (fieldValue === void 0)
                    return;
                var modify = typeof fields === "function"
                    ? fields
                    : fields[storeFieldName] || fields[fieldName];
                if (modify) {
                    var newValue = modify === delModifier ? DELETE :
                        modify(maybeDeepFreeze(fieldValue), __assign$2(__assign$2({}, sharedDetails_1), { fieldName: fieldName, storeFieldName: storeFieldName, storage: _this.getStorage(dataId, storeFieldName) }));
                    if (newValue === INVALIDATE) {
                        _this.group.dirty(dataId, storeFieldName);
                    }
                    else {
                        if (newValue === DELETE)
                            newValue = void 0;
                        if (newValue !== fieldValue) {
                            changedFields_1[storeFieldName] = newValue;
                            needToMerge_1 = true;
                            fieldValue = newValue;
                        }
                    }
                }
                if (fieldValue !== void 0) {
                    allDeleted_1 = false;
                }
            });
            if (needToMerge_1) {
                this.merge(dataId, changedFields_1);
                if (allDeleted_1) {
                    if (this instanceof Layer) {
                        this.data[dataId] = void 0;
                    }
                    else {
                        delete this.data[dataId];
                    }
                    this.group.dirty(dataId, "__exists");
                }
                return true;
            }
        }
        return false;
    };
    EntityStore.prototype.delete = function (dataId, fieldName, args) {
        var _a;
        var storeObject = this.lookup(dataId);
        if (storeObject) {
            var typename = this.getFieldValue(storeObject, "__typename");
            var storeFieldName = fieldName && args
                ? this.policies.getStoreFieldName({ typename: typename, fieldName: fieldName, args: args })
                : fieldName;
            return this.modify(dataId, storeFieldName ? (_a = {},
                _a[storeFieldName] = delModifier,
                _a) : delModifier);
        }
        return false;
    };
    EntityStore.prototype.evict = function (options, limit) {
        var evicted = false;
        if (options.id) {
            if (hasOwn.call(this.data, options.id)) {
                evicted = this.delete(options.id, options.fieldName, options.args);
            }
            if (this instanceof Layer && this !== limit) {
                evicted = this.parent.evict(options, limit) || evicted;
            }
            if (options.fieldName || evicted) {
                this.group.dirty(options.id, options.fieldName || "__exists");
            }
        }
        return evicted;
    };
    EntityStore.prototype.clear = function () {
        this.replace(null);
    };
    EntityStore.prototype.extract = function () {
        var _this = this;
        var obj = this.toObject();
        var extraRootIds = [];
        this.getRootIdSet().forEach(function (id) {
            if (!hasOwn.call(_this.policies.rootTypenamesById, id)) {
                extraRootIds.push(id);
            }
        });
        if (extraRootIds.length) {
            obj.__META = { extraRootIds: extraRootIds.sort() };
        }
        return obj;
    };
    EntityStore.prototype.replace = function (newData) {
        var _this = this;
        Object.keys(this.data).forEach(function (dataId) {
            if (!(newData && hasOwn.call(newData, dataId))) {
                _this.delete(dataId);
            }
        });
        if (newData) {
            var __META = newData.__META, rest_1 = __rest(newData, ["__META"]);
            Object.keys(rest_1).forEach(function (dataId) {
                _this.merge(dataId, rest_1[dataId]);
            });
            if (__META) {
                __META.extraRootIds.forEach(this.retain, this);
            }
        }
    };
    EntityStore.prototype.retain = function (rootId) {
        return this.rootIds[rootId] = (this.rootIds[rootId] || 0) + 1;
    };
    EntityStore.prototype.release = function (rootId) {
        if (this.rootIds[rootId] > 0) {
            var count = --this.rootIds[rootId];
            if (!count)
                delete this.rootIds[rootId];
            return count;
        }
        return 0;
    };
    EntityStore.prototype.getRootIdSet = function (ids) {
        if (ids === void 0) { ids = new Set(); }
        Object.keys(this.rootIds).forEach(ids.add, ids);
        if (this instanceof Layer) {
            this.parent.getRootIdSet(ids);
        }
        else {
            Object.keys(this.policies.rootTypenamesById).forEach(ids.add, ids);
        }
        return ids;
    };
    EntityStore.prototype.gc = function () {
        var _this = this;
        var ids = this.getRootIdSet();
        var snapshot = this.toObject();
        ids.forEach(function (id) {
            if (hasOwn.call(snapshot, id)) {
                Object.keys(_this.findChildRefIds(id)).forEach(ids.add, ids);
                delete snapshot[id];
            }
        });
        var idsToRemove = Object.keys(snapshot);
        if (idsToRemove.length) {
            var root_1 = this;
            while (root_1 instanceof Layer)
                root_1 = root_1.parent;
            idsToRemove.forEach(function (id) { return root_1.delete(id); });
        }
        return idsToRemove;
    };
    EntityStore.prototype.findChildRefIds = function (dataId) {
        if (!hasOwn.call(this.refs, dataId)) {
            var found_1 = this.refs[dataId] = Object.create(null);
            var root = this.data[dataId];
            if (!root)
                return found_1;
            var workSet_1 = new Set([root]);
            workSet_1.forEach(function (obj) {
                if (isReference(obj)) {
                    found_1[obj.__ref] = true;
                }
                if (isNonNullObject(obj)) {
                    Object.keys(obj).forEach(function (key) {
                        var child = obj[key];
                        if (isNonNullObject(child)) {
                            workSet_1.add(child);
                        }
                    });
                }
            });
        }
        return this.refs[dataId];
    };
    EntityStore.prototype.makeCacheKey = function () {
        return this.group.keyMaker.lookupArray(arguments);
    };
    return EntityStore;
}());
var CacheGroup = (function () {
    function CacheGroup(caching, parent) {
        if (parent === void 0) { parent = null; }
        this.caching = caching;
        this.parent = parent;
        this.d = null;
        this.resetCaching();
    }
    CacheGroup.prototype.resetCaching = function () {
        this.d = this.caching ? dep() : null;
        this.keyMaker = new Trie(canUseWeakMap);
    };
    CacheGroup.prototype.depend = function (dataId, storeFieldName) {
        if (this.d) {
            this.d(makeDepKey(dataId, storeFieldName));
            var fieldName = fieldNameFromStoreName(storeFieldName);
            if (fieldName !== storeFieldName) {
                this.d(makeDepKey(dataId, fieldName));
            }
            if (this.parent) {
                this.parent.depend(dataId, storeFieldName);
            }
        }
    };
    CacheGroup.prototype.dirty = function (dataId, storeFieldName) {
        if (this.d) {
            this.d.dirty(makeDepKey(dataId, storeFieldName), storeFieldName === "__exists" ? "forget" : "setDirty");
        }
    };
    return CacheGroup;
}());
function makeDepKey(dataId, storeFieldName) {
    return storeFieldName + '#' + dataId;
}
function maybeDependOnExistenceOfEntity(store, entityId) {
    if (supportsResultCaching(store)) {
        store.group.depend(entityId, "__exists");
    }
}
(function (EntityStore) {
    var Root = (function (_super) {
        __extends(Root, _super);
        function Root(_a) {
            var policies = _a.policies, _b = _a.resultCaching, resultCaching = _b === void 0 ? true : _b, seed = _a.seed;
            var _this = _super.call(this, policies, new CacheGroup(resultCaching)) || this;
            _this.stump = new Stump(_this);
            _this.storageTrie = new Trie(canUseWeakMap);
            if (seed)
                _this.replace(seed);
            return _this;
        }
        Root.prototype.addLayer = function (layerId, replay) {
            return this.stump.addLayer(layerId, replay);
        };
        Root.prototype.removeLayer = function () {
            return this;
        };
        Root.prototype.getStorage = function () {
            return this.storageTrie.lookupArray(arguments);
        };
        return Root;
    }(EntityStore));
    EntityStore.Root = Root;
})(EntityStore || (EntityStore = {}));
var Layer = (function (_super) {
    __extends(Layer, _super);
    function Layer(id, parent, replay, group) {
        var _this = _super.call(this, parent.policies, group) || this;
        _this.id = id;
        _this.parent = parent;
        _this.replay = replay;
        _this.group = group;
        replay(_this);
        return _this;
    }
    Layer.prototype.addLayer = function (layerId, replay) {
        return new Layer(layerId, this, replay, this.group);
    };
    Layer.prototype.removeLayer = function (layerId) {
        var _this = this;
        var parent = this.parent.removeLayer(layerId);
        if (layerId === this.id) {
            if (this.group.caching) {
                Object.keys(this.data).forEach(function (dataId) {
                    var ownStoreObject = _this.data[dataId];
                    var parentStoreObject = parent["lookup"](dataId);
                    if (!parentStoreObject) {
                        _this.delete(dataId);
                    }
                    else if (!ownStoreObject) {
                        _this.group.dirty(dataId, "__exists");
                        Object.keys(parentStoreObject).forEach(function (storeFieldName) {
                            _this.group.dirty(dataId, storeFieldName);
                        });
                    }
                    else if (ownStoreObject !== parentStoreObject) {
                        Object.keys(ownStoreObject).forEach(function (storeFieldName) {
                            if (!equal(ownStoreObject[storeFieldName], parentStoreObject[storeFieldName])) {
                                _this.group.dirty(dataId, storeFieldName);
                            }
                        });
                    }
                });
            }
            return parent;
        }
        if (parent === this.parent)
            return this;
        return parent.addLayer(this.id, this.replay);
    };
    Layer.prototype.toObject = function () {
        return __assign$2(__assign$2({}, this.parent.toObject()), this.data);
    };
    Layer.prototype.findChildRefIds = function (dataId) {
        var fromParent = this.parent.findChildRefIds(dataId);
        return hasOwn.call(this.data, dataId) ? __assign$2(__assign$2({}, fromParent), _super.prototype.findChildRefIds.call(this, dataId)) : fromParent;
    };
    Layer.prototype.getStorage = function () {
        var p = this.parent;
        while (p.parent)
            p = p.parent;
        return p.getStorage.apply(p, arguments);
    };
    return Layer;
}(EntityStore));
var Stump = (function (_super) {
    __extends(Stump, _super);
    function Stump(root) {
        return _super.call(this, "EntityStore.Stump", root, function () { }, new CacheGroup(root.group.caching, root.group)) || this;
    }
    Stump.prototype.removeLayer = function () {
        return this;
    };
    Stump.prototype.merge = function () {
        return this.parent.merge.apply(this.parent, arguments);
    };
    return Stump;
}(Layer));
function storeObjectReconciler(existingObject, incomingObject, property) {
    var existingValue = existingObject[property];
    var incomingValue = incomingObject[property];
    return equal(existingValue, incomingValue) ? existingValue : incomingValue;
}
function supportsResultCaching(store) {
    return !!(store instanceof EntityStore && store.group.caching);
}

function shallowCopy(value) {
    if (isNonNullObject(value)) {
        return isArray(value)
            ? value.slice(0)
            : __assign$2({ __proto__: Object.getPrototypeOf(value) }, value);
    }
    return value;
}
var ObjectCanon = (function () {
    function ObjectCanon() {
        this.known = new (canUseWeakSet ? WeakSet : Set)();
        this.pool = new Trie(canUseWeakMap);
        this.passes = new WeakMap();
        this.keysByJSON = new Map();
        this.empty = this.admit({});
    }
    ObjectCanon.prototype.isKnown = function (value) {
        return isNonNullObject(value) && this.known.has(value);
    };
    ObjectCanon.prototype.pass = function (value) {
        if (isNonNullObject(value)) {
            var copy = shallowCopy(value);
            this.passes.set(copy, value);
            return copy;
        }
        return value;
    };
    ObjectCanon.prototype.admit = function (value) {
        var _this = this;
        if (isNonNullObject(value)) {
            var original = this.passes.get(value);
            if (original)
                return original;
            var proto = Object.getPrototypeOf(value);
            switch (proto) {
                case Array.prototype: {
                    if (this.known.has(value))
                        return value;
                    var array = value.map(this.admit, this);
                    var node = this.pool.lookupArray(array);
                    if (!node.array) {
                        this.known.add(node.array = array);
                        if (__DEV__) {
                            Object.freeze(array);
                        }
                    }
                    return node.array;
                }
                case null:
                case Object.prototype: {
                    if (this.known.has(value))
                        return value;
                    var proto_1 = Object.getPrototypeOf(value);
                    var array_1 = [proto_1];
                    var keys = this.sortedKeys(value);
                    array_1.push(keys.json);
                    var firstValueIndex_1 = array_1.length;
                    keys.sorted.forEach(function (key) {
                        array_1.push(_this.admit(value[key]));
                    });
                    var node = this.pool.lookupArray(array_1);
                    if (!node.object) {
                        var obj_1 = node.object = Object.create(proto_1);
                        this.known.add(obj_1);
                        keys.sorted.forEach(function (key, i) {
                            obj_1[key] = array_1[firstValueIndex_1 + i];
                        });
                        if (__DEV__) {
                            Object.freeze(obj_1);
                        }
                    }
                    return node.object;
                }
            }
        }
        return value;
    };
    ObjectCanon.prototype.sortedKeys = function (obj) {
        var keys = Object.keys(obj);
        var node = this.pool.lookupArray(keys);
        if (!node.keys) {
            keys.sort();
            var json = JSON.stringify(keys);
            if (!(node.keys = this.keysByJSON.get(json))) {
                this.keysByJSON.set(json, node.keys = { sorted: keys, json: json });
            }
        }
        return node.keys;
    };
    return ObjectCanon;
}());
var canonicalStringify = Object.assign(function (value) {
    if (isNonNullObject(value)) {
        if (stringifyCanon === void 0) {
            resetCanonicalStringify();
        }
        var canonical = stringifyCanon.admit(value);
        var json = stringifyCache.get(canonical);
        if (json === void 0) {
            stringifyCache.set(canonical, json = JSON.stringify(canonical));
        }
        return json;
    }
    return JSON.stringify(value);
}, {
    reset: resetCanonicalStringify,
});
var stringifyCanon;
var stringifyCache;
function resetCanonicalStringify() {
    stringifyCanon = new ObjectCanon;
    stringifyCache = new (canUseWeakMap ? WeakMap : Map)();
}

function execSelectionSetKeyArgs(options) {
    return [
        options.selectionSet,
        options.objectOrReference,
        options.context,
        options.context.canonizeResults,
    ];
}
var StoreReader = (function () {
    function StoreReader(config) {
        var _this = this;
        this.knownResults = new (canUseWeakMap ? WeakMap : Map)();
        this.config = compact(config, {
            addTypename: config.addTypename !== false,
            canonizeResults: shouldCanonizeResults(config),
        });
        this.canon = config.canon || new ObjectCanon;
        this.executeSelectionSet = wrap(function (options) {
            var _a;
            var canonizeResults = options.context.canonizeResults;
            var peekArgs = execSelectionSetKeyArgs(options);
            peekArgs[3] = !canonizeResults;
            var other = (_a = _this.executeSelectionSet).peek.apply(_a, peekArgs);
            if (other) {
                if (canonizeResults) {
                    return __assign$2(__assign$2({}, other), { result: _this.canon.admit(other.result) });
                }
                return other;
            }
            maybeDependOnExistenceOfEntity(options.context.store, options.enclosingRef.__ref);
            return _this.execSelectionSetImpl(options);
        }, {
            max: this.config.resultCacheMaxSize,
            keyArgs: execSelectionSetKeyArgs,
            makeCacheKey: function (selectionSet, parent, context, canonizeResults) {
                if (supportsResultCaching(context.store)) {
                    return context.store.makeCacheKey(selectionSet, isReference(parent) ? parent.__ref : parent, context.varString, canonizeResults);
                }
            }
        });
        this.executeSubSelectedArray = wrap(function (options) {
            maybeDependOnExistenceOfEntity(options.context.store, options.enclosingRef.__ref);
            return _this.execSubSelectedArrayImpl(options);
        }, {
            max: this.config.resultCacheMaxSize,
            makeCacheKey: function (_a) {
                var field = _a.field, array = _a.array, context = _a.context;
                if (supportsResultCaching(context.store)) {
                    return context.store.makeCacheKey(field, array, context.varString);
                }
            }
        });
    }
    StoreReader.prototype.resetCanon = function () {
        this.canon = new ObjectCanon;
    };
    StoreReader.prototype.diffQueryAgainstStore = function (_a) {
        var store = _a.store, query = _a.query, _b = _a.rootId, rootId = _b === void 0 ? 'ROOT_QUERY' : _b, variables = _a.variables, _c = _a.returnPartialData, returnPartialData = _c === void 0 ? true : _c, _d = _a.canonizeResults, canonizeResults = _d === void 0 ? this.config.canonizeResults : _d;
        var policies = this.config.cache.policies;
        variables = __assign$2(__assign$2({}, getDefaultValues(getQueryDefinition(query))), variables);
        var rootRef = makeReference(rootId);
        var execResult = this.executeSelectionSet({
            selectionSet: getMainDefinition(query).selectionSet,
            objectOrReference: rootRef,
            enclosingRef: rootRef,
            context: __assign$2({ store: store, query: query, policies: policies, variables: variables, varString: canonicalStringify(variables), canonizeResults: canonizeResults }, extractFragmentContext(query, this.config.fragments)),
        });
        var missing;
        if (execResult.missing) {
            missing = [new MissingFieldError(firstMissing(execResult.missing), execResult.missing, query, variables)];
            if (!returnPartialData) {
                throw missing[0];
            }
        }
        return {
            result: execResult.result,
            complete: !missing,
            missing: missing,
        };
    };
    StoreReader.prototype.isFresh = function (result, parent, selectionSet, context) {
        if (supportsResultCaching(context.store) &&
            this.knownResults.get(result) === selectionSet) {
            var latest = this.executeSelectionSet.peek(selectionSet, parent, context, this.canon.isKnown(result));
            if (latest && result === latest.result) {
                return true;
            }
        }
        return false;
    };
    StoreReader.prototype.execSelectionSetImpl = function (_a) {
        var _this = this;
        var selectionSet = _a.selectionSet, objectOrReference = _a.objectOrReference, enclosingRef = _a.enclosingRef, context = _a.context;
        if (isReference(objectOrReference) &&
            !context.policies.rootTypenamesById[objectOrReference.__ref] &&
            !context.store.has(objectOrReference.__ref)) {
            return {
                result: this.canon.empty,
                missing: "Dangling reference to missing ".concat(objectOrReference.__ref, " object"),
            };
        }
        var variables = context.variables, policies = context.policies, store = context.store;
        var typename = store.getFieldValue(objectOrReference, "__typename");
        var objectsToMerge = [];
        var missing;
        var missingMerger = new DeepMerger();
        if (this.config.addTypename &&
            typeof typename === "string" &&
            !policies.rootIdsByTypename[typename]) {
            objectsToMerge.push({ __typename: typename });
        }
        function handleMissing(result, resultName) {
            var _a;
            if (result.missing) {
                missing = missingMerger.merge(missing, (_a = {}, _a[resultName] = result.missing, _a));
            }
            return result.result;
        }
        var workSet = new Set(selectionSet.selections);
        workSet.forEach(function (selection) {
            var _a, _b;
            if (!shouldInclude(selection, variables))
                return;
            if (isField(selection)) {
                var fieldValue = policies.readField({
                    fieldName: selection.name.value,
                    field: selection,
                    variables: context.variables,
                    from: objectOrReference,
                }, context);
                var resultName = resultKeyNameFromField(selection);
                if (fieldValue === void 0) {
                    if (!addTypenameToDocument.added(selection)) {
                        missing = missingMerger.merge(missing, (_a = {},
                            _a[resultName] = "Can't find field '".concat(selection.name.value, "' on ").concat(isReference(objectOrReference)
                                ? objectOrReference.__ref + " object"
                                : "object " + JSON.stringify(objectOrReference, null, 2)),
                            _a));
                    }
                }
                else if (isArray(fieldValue)) {
                    fieldValue = handleMissing(_this.executeSubSelectedArray({
                        field: selection,
                        array: fieldValue,
                        enclosingRef: enclosingRef,
                        context: context,
                    }), resultName);
                }
                else if (!selection.selectionSet) {
                    if (context.canonizeResults) {
                        fieldValue = _this.canon.pass(fieldValue);
                    }
                }
                else if (fieldValue != null) {
                    fieldValue = handleMissing(_this.executeSelectionSet({
                        selectionSet: selection.selectionSet,
                        objectOrReference: fieldValue,
                        enclosingRef: isReference(fieldValue) ? fieldValue : enclosingRef,
                        context: context,
                    }), resultName);
                }
                if (fieldValue !== void 0) {
                    objectsToMerge.push((_b = {}, _b[resultName] = fieldValue, _b));
                }
            }
            else {
                var fragment = getFragmentFromSelection(selection, context.lookupFragment);
                if (!fragment && selection.kind === Kind$1.FRAGMENT_SPREAD) {
                    throw __DEV__ ? new InvariantError("No fragment named ".concat(selection.name.value)) : new InvariantError(5);
                }
                if (fragment && policies.fragmentMatches(fragment, typename)) {
                    fragment.selectionSet.selections.forEach(workSet.add, workSet);
                }
            }
        });
        var result = mergeDeepArray(objectsToMerge);
        var finalResult = { result: result, missing: missing };
        var frozen = context.canonizeResults
            ? this.canon.admit(finalResult)
            : maybeDeepFreeze(finalResult);
        if (frozen.result) {
            this.knownResults.set(frozen.result, selectionSet);
        }
        return frozen;
    };
    StoreReader.prototype.execSubSelectedArrayImpl = function (_a) {
        var _this = this;
        var field = _a.field, array = _a.array, enclosingRef = _a.enclosingRef, context = _a.context;
        var missing;
        var missingMerger = new DeepMerger();
        function handleMissing(childResult, i) {
            var _a;
            if (childResult.missing) {
                missing = missingMerger.merge(missing, (_a = {}, _a[i] = childResult.missing, _a));
            }
            return childResult.result;
        }
        if (field.selectionSet) {
            array = array.filter(context.store.canRead);
        }
        array = array.map(function (item, i) {
            if (item === null) {
                return null;
            }
            if (isArray(item)) {
                return handleMissing(_this.executeSubSelectedArray({
                    field: field,
                    array: item,
                    enclosingRef: enclosingRef,
                    context: context,
                }), i);
            }
            if (field.selectionSet) {
                return handleMissing(_this.executeSelectionSet({
                    selectionSet: field.selectionSet,
                    objectOrReference: item,
                    enclosingRef: isReference(item) ? item : enclosingRef,
                    context: context,
                }), i);
            }
            if (__DEV__) {
                assertSelectionSetForIdValue(context.store, field, item);
            }
            return item;
        });
        return {
            result: context.canonizeResults ? this.canon.admit(array) : array,
            missing: missing,
        };
    };
    return StoreReader;
}());
function firstMissing(tree) {
    try {
        JSON.stringify(tree, function (_, value) {
            if (typeof value === "string")
                throw value;
            return value;
        });
    }
    catch (result) {
        return result;
    }
}
function assertSelectionSetForIdValue(store, field, fieldValue) {
    if (!field.selectionSet) {
        var workSet_1 = new Set([fieldValue]);
        workSet_1.forEach(function (value) {
            if (isNonNullObject(value)) {
                __DEV__ ? invariant$2(!isReference(value), "Missing selection set for object of type ".concat(getTypenameFromStoreObject(store, value), " returned for query field ").concat(field.name.value)) : invariant$2(!isReference(value), 6);
                Object.values(value).forEach(workSet_1.add, workSet_1);
            }
        });
    }
}

var cacheSlot = new Slot();
var cacheInfoMap = new WeakMap();
function getCacheInfo(cache) {
    var info = cacheInfoMap.get(cache);
    if (!info) {
        cacheInfoMap.set(cache, info = {
            vars: new Set,
            dep: dep(),
        });
    }
    return info;
}
function forgetCache(cache) {
    getCacheInfo(cache).vars.forEach(function (rv) { return rv.forgetCache(cache); });
}
function recallCache(cache) {
    getCacheInfo(cache).vars.forEach(function (rv) { return rv.attachCache(cache); });
}
function makeVar(value) {
    var caches = new Set();
    var listeners = new Set();
    var rv = function (newValue) {
        if (arguments.length > 0) {
            if (value !== newValue) {
                value = newValue;
                caches.forEach(function (cache) {
                    getCacheInfo(cache).dep.dirty(rv);
                    broadcast(cache);
                });
                var oldListeners = Array.from(listeners);
                listeners.clear();
                oldListeners.forEach(function (listener) { return listener(value); });
            }
        }
        else {
            var cache = cacheSlot.getValue();
            if (cache) {
                attach(cache);
                getCacheInfo(cache).dep(rv);
            }
        }
        return value;
    };
    rv.onNextChange = function (listener) {
        listeners.add(listener);
        return function () {
            listeners.delete(listener);
        };
    };
    var attach = rv.attachCache = function (cache) {
        caches.add(cache);
        getCacheInfo(cache).vars.add(rv);
        return rv;
    };
    rv.forgetCache = function (cache) { return caches.delete(cache); };
    return rv;
}
function broadcast(cache) {
    if (cache.broadcastWatches) {
        cache.broadcastWatches();
    }
}

var specifierInfoCache = Object.create(null);
function lookupSpecifierInfo(spec) {
    var cacheKey = JSON.stringify(spec);
    return specifierInfoCache[cacheKey] ||
        (specifierInfoCache[cacheKey] = Object.create(null));
}
function keyFieldsFnFromSpecifier(specifier) {
    var info = lookupSpecifierInfo(specifier);
    return info.keyFieldsFn || (info.keyFieldsFn = function (object, context) {
        var extract = function (from, key) { return context.readField(key, from); };
        var keyObject = context.keyObject = collectSpecifierPaths(specifier, function (schemaKeyPath) {
            var extracted = extractKeyPath(context.storeObject, schemaKeyPath, extract);
            if (extracted === void 0 &&
                object !== context.storeObject &&
                hasOwn.call(object, schemaKeyPath[0])) {
                extracted = extractKeyPath(object, schemaKeyPath, extractKey);
            }
            __DEV__ ? invariant$2(extracted !== void 0, "Missing field '".concat(schemaKeyPath.join('.'), "' while extracting keyFields from ").concat(JSON.stringify(object))) : invariant$2(extracted !== void 0, 2);
            return extracted;
        });
        return "".concat(context.typename, ":").concat(JSON.stringify(keyObject));
    });
}
function keyArgsFnFromSpecifier(specifier) {
    var info = lookupSpecifierInfo(specifier);
    return info.keyArgsFn || (info.keyArgsFn = function (args, _a) {
        var field = _a.field, variables = _a.variables, fieldName = _a.fieldName;
        var collected = collectSpecifierPaths(specifier, function (keyPath) {
            var firstKey = keyPath[0];
            var firstChar = firstKey.charAt(0);
            if (firstChar === "@") {
                if (field && isNonEmptyArray(field.directives)) {
                    var directiveName_1 = firstKey.slice(1);
                    var d = field.directives.find(function (d) { return d.name.value === directiveName_1; });
                    var directiveArgs = d && argumentsObjectFromField(d, variables);
                    return directiveArgs && extractKeyPath(directiveArgs, keyPath.slice(1));
                }
                return;
            }
            if (firstChar === "$") {
                var variableName = firstKey.slice(1);
                if (variables && hasOwn.call(variables, variableName)) {
                    var varKeyPath = keyPath.slice(0);
                    varKeyPath[0] = variableName;
                    return extractKeyPath(variables, varKeyPath);
                }
                return;
            }
            if (args) {
                return extractKeyPath(args, keyPath);
            }
        });
        var suffix = JSON.stringify(collected);
        if (args || suffix !== "{}") {
            fieldName += ":" + suffix;
        }
        return fieldName;
    });
}
function collectSpecifierPaths(specifier, extractor) {
    var merger = new DeepMerger;
    return getSpecifierPaths(specifier).reduce(function (collected, path) {
        var _a;
        var toMerge = extractor(path);
        if (toMerge !== void 0) {
            for (var i = path.length - 1; i >= 0; --i) {
                toMerge = (_a = {}, _a[path[i]] = toMerge, _a);
            }
            collected = merger.merge(collected, toMerge);
        }
        return collected;
    }, Object.create(null));
}
function getSpecifierPaths(spec) {
    var info = lookupSpecifierInfo(spec);
    if (!info.paths) {
        var paths_1 = info.paths = [];
        var currentPath_1 = [];
        spec.forEach(function (s, i) {
            if (isArray(s)) {
                getSpecifierPaths(s).forEach(function (p) { return paths_1.push(currentPath_1.concat(p)); });
                currentPath_1.length = 0;
            }
            else {
                currentPath_1.push(s);
                if (!isArray(spec[i + 1])) {
                    paths_1.push(currentPath_1.slice(0));
                    currentPath_1.length = 0;
                }
            }
        });
    }
    return info.paths;
}
function extractKey(object, key) {
    return object[key];
}
function extractKeyPath(object, path, extract) {
    extract = extract || extractKey;
    return normalize$1(path.reduce(function reducer(obj, key) {
        return isArray(obj)
            ? obj.map(function (child) { return reducer(child, key); })
            : obj && extract(obj, key);
    }, object));
}
function normalize$1(value) {
    if (isNonNullObject(value)) {
        if (isArray(value)) {
            return value.map(normalize$1);
        }
        return collectSpecifierPaths(Object.keys(value).sort(), function (path) { return extractKeyPath(value, path); });
    }
    return value;
}

getStoreKeyName.setStringify(canonicalStringify);
function argsFromFieldSpecifier(spec) {
    return spec.args !== void 0 ? spec.args :
        spec.field ? argumentsObjectFromField(spec.field, spec.variables) : null;
}
var nullKeyFieldsFn = function () { return void 0; };
var simpleKeyArgsFn = function (_args, context) { return context.fieldName; };
var mergeTrueFn = function (existing, incoming, _a) {
    var mergeObjects = _a.mergeObjects;
    return mergeObjects(existing, incoming);
};
var mergeFalseFn = function (_, incoming) { return incoming; };
var Policies = (function () {
    function Policies(config) {
        this.config = config;
        this.typePolicies = Object.create(null);
        this.toBeAdded = Object.create(null);
        this.supertypeMap = new Map();
        this.fuzzySubtypes = new Map();
        this.rootIdsByTypename = Object.create(null);
        this.rootTypenamesById = Object.create(null);
        this.usingPossibleTypes = false;
        this.config = __assign$2({ dataIdFromObject: defaultDataIdFromObject }, config);
        this.cache = this.config.cache;
        this.setRootTypename("Query");
        this.setRootTypename("Mutation");
        this.setRootTypename("Subscription");
        if (config.possibleTypes) {
            this.addPossibleTypes(config.possibleTypes);
        }
        if (config.typePolicies) {
            this.addTypePolicies(config.typePolicies);
        }
    }
    Policies.prototype.identify = function (object, partialContext) {
        var _a;
        var policies = this;
        var typename = partialContext && (partialContext.typename ||
            ((_a = partialContext.storeObject) === null || _a === void 0 ? void 0 : _a.__typename)) || object.__typename;
        if (typename === this.rootTypenamesById.ROOT_QUERY) {
            return ["ROOT_QUERY"];
        }
        var storeObject = partialContext && partialContext.storeObject || object;
        var context = __assign$2(__assign$2({}, partialContext), { typename: typename, storeObject: storeObject, readField: partialContext && partialContext.readField || function () {
                var options = normalizeReadFieldOptions(arguments, storeObject);
                return policies.readField(options, {
                    store: policies.cache["data"],
                    variables: options.variables,
                });
            } });
        var id;
        var policy = typename && this.getTypePolicy(typename);
        var keyFn = policy && policy.keyFn || this.config.dataIdFromObject;
        while (keyFn) {
            var specifierOrId = keyFn(object, context);
            if (isArray(specifierOrId)) {
                keyFn = keyFieldsFnFromSpecifier(specifierOrId);
            }
            else {
                id = specifierOrId;
                break;
            }
        }
        id = id ? String(id) : void 0;
        return context.keyObject ? [id, context.keyObject] : [id];
    };
    Policies.prototype.addTypePolicies = function (typePolicies) {
        var _this = this;
        Object.keys(typePolicies).forEach(function (typename) {
            var _a = typePolicies[typename], queryType = _a.queryType, mutationType = _a.mutationType, subscriptionType = _a.subscriptionType, incoming = __rest(_a, ["queryType", "mutationType", "subscriptionType"]);
            if (queryType)
                _this.setRootTypename("Query", typename);
            if (mutationType)
                _this.setRootTypename("Mutation", typename);
            if (subscriptionType)
                _this.setRootTypename("Subscription", typename);
            if (hasOwn.call(_this.toBeAdded, typename)) {
                _this.toBeAdded[typename].push(incoming);
            }
            else {
                _this.toBeAdded[typename] = [incoming];
            }
        });
    };
    Policies.prototype.updateTypePolicy = function (typename, incoming) {
        var _this = this;
        var existing = this.getTypePolicy(typename);
        var keyFields = incoming.keyFields, fields = incoming.fields;
        function setMerge(existing, merge) {
            existing.merge =
                typeof merge === "function" ? merge :
                    merge === true ? mergeTrueFn :
                        merge === false ? mergeFalseFn :
                            existing.merge;
        }
        setMerge(existing, incoming.merge);
        existing.keyFn =
            keyFields === false ? nullKeyFieldsFn :
                isArray(keyFields) ? keyFieldsFnFromSpecifier(keyFields) :
                    typeof keyFields === "function" ? keyFields :
                        existing.keyFn;
        if (fields) {
            Object.keys(fields).forEach(function (fieldName) {
                var existing = _this.getFieldPolicy(typename, fieldName, true);
                var incoming = fields[fieldName];
                if (typeof incoming === "function") {
                    existing.read = incoming;
                }
                else {
                    var keyArgs = incoming.keyArgs, read = incoming.read, merge = incoming.merge;
                    existing.keyFn =
                        keyArgs === false ? simpleKeyArgsFn :
                            isArray(keyArgs) ? keyArgsFnFromSpecifier(keyArgs) :
                                typeof keyArgs === "function" ? keyArgs :
                                    existing.keyFn;
                    if (typeof read === "function") {
                        existing.read = read;
                    }
                    setMerge(existing, merge);
                }
                if (existing.read && existing.merge) {
                    existing.keyFn = existing.keyFn || simpleKeyArgsFn;
                }
            });
        }
    };
    Policies.prototype.setRootTypename = function (which, typename) {
        if (typename === void 0) { typename = which; }
        var rootId = "ROOT_" + which.toUpperCase();
        var old = this.rootTypenamesById[rootId];
        if (typename !== old) {
            __DEV__ ? invariant$2(!old || old === which, "Cannot change root ".concat(which, " __typename more than once")) : invariant$2(!old || old === which, 3);
            if (old)
                delete this.rootIdsByTypename[old];
            this.rootIdsByTypename[typename] = rootId;
            this.rootTypenamesById[rootId] = typename;
        }
    };
    Policies.prototype.addPossibleTypes = function (possibleTypes) {
        var _this = this;
        this.usingPossibleTypes = true;
        Object.keys(possibleTypes).forEach(function (supertype) {
            _this.getSupertypeSet(supertype, true);
            possibleTypes[supertype].forEach(function (subtype) {
                _this.getSupertypeSet(subtype, true).add(supertype);
                var match = subtype.match(TypeOrFieldNameRegExp);
                if (!match || match[0] !== subtype) {
                    _this.fuzzySubtypes.set(subtype, new RegExp(subtype));
                }
            });
        });
    };
    Policies.prototype.getTypePolicy = function (typename) {
        var _this = this;
        if (!hasOwn.call(this.typePolicies, typename)) {
            var policy_1 = this.typePolicies[typename] = Object.create(null);
            policy_1.fields = Object.create(null);
            var supertypes = this.supertypeMap.get(typename);
            if (supertypes && supertypes.size) {
                supertypes.forEach(function (supertype) {
                    var _a = _this.getTypePolicy(supertype), fields = _a.fields, rest = __rest(_a, ["fields"]);
                    Object.assign(policy_1, rest);
                    Object.assign(policy_1.fields, fields);
                });
            }
        }
        var inbox = this.toBeAdded[typename];
        if (inbox && inbox.length) {
            inbox.splice(0).forEach(function (policy) {
                _this.updateTypePolicy(typename, policy);
            });
        }
        return this.typePolicies[typename];
    };
    Policies.prototype.getFieldPolicy = function (typename, fieldName, createIfMissing) {
        if (typename) {
            var fieldPolicies = this.getTypePolicy(typename).fields;
            return fieldPolicies[fieldName] || (createIfMissing && (fieldPolicies[fieldName] = Object.create(null)));
        }
    };
    Policies.prototype.getSupertypeSet = function (subtype, createIfMissing) {
        var supertypeSet = this.supertypeMap.get(subtype);
        if (!supertypeSet && createIfMissing) {
            this.supertypeMap.set(subtype, supertypeSet = new Set());
        }
        return supertypeSet;
    };
    Policies.prototype.fragmentMatches = function (fragment, typename, result, variables) {
        var _this = this;
        if (!fragment.typeCondition)
            return true;
        if (!typename)
            return false;
        var supertype = fragment.typeCondition.name.value;
        if (typename === supertype)
            return true;
        if (this.usingPossibleTypes &&
            this.supertypeMap.has(supertype)) {
            var typenameSupertypeSet = this.getSupertypeSet(typename, true);
            var workQueue_1 = [typenameSupertypeSet];
            var maybeEnqueue_1 = function (subtype) {
                var supertypeSet = _this.getSupertypeSet(subtype, false);
                if (supertypeSet &&
                    supertypeSet.size &&
                    workQueue_1.indexOf(supertypeSet) < 0) {
                    workQueue_1.push(supertypeSet);
                }
            };
            var needToCheckFuzzySubtypes = !!(result && this.fuzzySubtypes.size);
            var checkingFuzzySubtypes = false;
            for (var i = 0; i < workQueue_1.length; ++i) {
                var supertypeSet = workQueue_1[i];
                if (supertypeSet.has(supertype)) {
                    if (!typenameSupertypeSet.has(supertype)) {
                        if (checkingFuzzySubtypes) {
                            __DEV__ && invariant$2.warn("Inferring subtype ".concat(typename, " of supertype ").concat(supertype));
                        }
                        typenameSupertypeSet.add(supertype);
                    }
                    return true;
                }
                supertypeSet.forEach(maybeEnqueue_1);
                if (needToCheckFuzzySubtypes &&
                    i === workQueue_1.length - 1 &&
                    selectionSetMatchesResult(fragment.selectionSet, result, variables)) {
                    needToCheckFuzzySubtypes = false;
                    checkingFuzzySubtypes = true;
                    this.fuzzySubtypes.forEach(function (regExp, fuzzyString) {
                        var match = typename.match(regExp);
                        if (match && match[0] === typename) {
                            maybeEnqueue_1(fuzzyString);
                        }
                    });
                }
            }
        }
        return false;
    };
    Policies.prototype.hasKeyArgs = function (typename, fieldName) {
        var policy = this.getFieldPolicy(typename, fieldName, false);
        return !!(policy && policy.keyFn);
    };
    Policies.prototype.getStoreFieldName = function (fieldSpec) {
        var typename = fieldSpec.typename, fieldName = fieldSpec.fieldName;
        var policy = this.getFieldPolicy(typename, fieldName, false);
        var storeFieldName;
        var keyFn = policy && policy.keyFn;
        if (keyFn && typename) {
            var context = {
                typename: typename,
                fieldName: fieldName,
                field: fieldSpec.field || null,
                variables: fieldSpec.variables,
            };
            var args = argsFromFieldSpecifier(fieldSpec);
            while (keyFn) {
                var specifierOrString = keyFn(args, context);
                if (isArray(specifierOrString)) {
                    keyFn = keyArgsFnFromSpecifier(specifierOrString);
                }
                else {
                    storeFieldName = specifierOrString || fieldName;
                    break;
                }
            }
        }
        if (storeFieldName === void 0) {
            storeFieldName = fieldSpec.field
                ? storeKeyNameFromField(fieldSpec.field, fieldSpec.variables)
                : getStoreKeyName(fieldName, argsFromFieldSpecifier(fieldSpec));
        }
        if (storeFieldName === false) {
            return fieldName;
        }
        return fieldName === fieldNameFromStoreName(storeFieldName)
            ? storeFieldName
            : fieldName + ":" + storeFieldName;
    };
    Policies.prototype.readField = function (options, context) {
        var objectOrReference = options.from;
        if (!objectOrReference)
            return;
        var nameOrField = options.field || options.fieldName;
        if (!nameOrField)
            return;
        if (options.typename === void 0) {
            var typename = context.store.getFieldValue(objectOrReference, "__typename");
            if (typename)
                options.typename = typename;
        }
        var storeFieldName = this.getStoreFieldName(options);
        var fieldName = fieldNameFromStoreName(storeFieldName);
        var existing = context.store.getFieldValue(objectOrReference, storeFieldName);
        var policy = this.getFieldPolicy(options.typename, fieldName, false);
        var read = policy && policy.read;
        if (read) {
            var readOptions = makeFieldFunctionOptions(this, objectOrReference, options, context, context.store.getStorage(isReference(objectOrReference)
                ? objectOrReference.__ref
                : objectOrReference, storeFieldName));
            return cacheSlot.withValue(this.cache, read, [existing, readOptions]);
        }
        return existing;
    };
    Policies.prototype.getReadFunction = function (typename, fieldName) {
        var policy = this.getFieldPolicy(typename, fieldName, false);
        return policy && policy.read;
    };
    Policies.prototype.getMergeFunction = function (parentTypename, fieldName, childTypename) {
        var policy = this.getFieldPolicy(parentTypename, fieldName, false);
        var merge = policy && policy.merge;
        if (!merge && childTypename) {
            policy = this.getTypePolicy(childTypename);
            merge = policy && policy.merge;
        }
        return merge;
    };
    Policies.prototype.runMergeFunction = function (existing, incoming, _a, context, storage) {
        var field = _a.field, typename = _a.typename, merge = _a.merge;
        if (merge === mergeTrueFn) {
            return makeMergeObjectsFunction(context.store)(existing, incoming);
        }
        if (merge === mergeFalseFn) {
            return incoming;
        }
        if (context.overwrite) {
            existing = void 0;
        }
        return merge(existing, incoming, makeFieldFunctionOptions(this, void 0, { typename: typename, fieldName: field.name.value, field: field, variables: context.variables }, context, storage || Object.create(null)));
    };
    return Policies;
}());
function makeFieldFunctionOptions(policies, objectOrReference, fieldSpec, context, storage) {
    var storeFieldName = policies.getStoreFieldName(fieldSpec);
    var fieldName = fieldNameFromStoreName(storeFieldName);
    var variables = fieldSpec.variables || context.variables;
    var _a = context.store, toReference = _a.toReference, canRead = _a.canRead;
    return {
        args: argsFromFieldSpecifier(fieldSpec),
        field: fieldSpec.field || null,
        fieldName: fieldName,
        storeFieldName: storeFieldName,
        variables: variables,
        isReference: isReference,
        toReference: toReference,
        storage: storage,
        cache: policies.cache,
        canRead: canRead,
        readField: function () {
            return policies.readField(normalizeReadFieldOptions(arguments, objectOrReference, variables), context);
        },
        mergeObjects: makeMergeObjectsFunction(context.store),
    };
}
function normalizeReadFieldOptions(readFieldArgs, objectOrReference, variables) {
    var fieldNameOrOptions = readFieldArgs[0], from = readFieldArgs[1], argc = readFieldArgs.length;
    var options;
    if (typeof fieldNameOrOptions === "string") {
        options = {
            fieldName: fieldNameOrOptions,
            from: argc > 1 ? from : objectOrReference,
        };
    }
    else {
        options = __assign$2({}, fieldNameOrOptions);
        if (!hasOwn.call(options, "from")) {
            options.from = objectOrReference;
        }
    }
    if (__DEV__ && options.from === void 0) {
        __DEV__ && invariant$2.warn("Undefined 'from' passed to readField with arguments ".concat(stringifyForDisplay(Array.from(readFieldArgs))));
    }
    if (void 0 === options.variables) {
        options.variables = variables;
    }
    return options;
}
function makeMergeObjectsFunction(store) {
    return function mergeObjects(existing, incoming) {
        if (isArray(existing) || isArray(incoming)) {
            throw __DEV__ ? new InvariantError("Cannot automatically merge arrays") : new InvariantError(4);
        }
        if (isNonNullObject(existing) &&
            isNonNullObject(incoming)) {
            var eType = store.getFieldValue(existing, "__typename");
            var iType = store.getFieldValue(incoming, "__typename");
            var typesDiffer = eType && iType && eType !== iType;
            if (typesDiffer) {
                return incoming;
            }
            if (isReference(existing) &&
                storeValueIsStoreObject(incoming)) {
                store.merge(existing.__ref, incoming);
                return existing;
            }
            if (storeValueIsStoreObject(existing) &&
                isReference(incoming)) {
                store.merge(existing, incoming.__ref);
                return incoming;
            }
            if (storeValueIsStoreObject(existing) &&
                storeValueIsStoreObject(incoming)) {
                return __assign$2(__assign$2({}, existing), incoming);
            }
        }
        return incoming;
    };
}

function getContextFlavor(context, clientOnly, deferred) {
    var key = "".concat(clientOnly).concat(deferred);
    var flavored = context.flavors.get(key);
    if (!flavored) {
        context.flavors.set(key, flavored = (context.clientOnly === clientOnly &&
            context.deferred === deferred) ? context : __assign$2(__assign$2({}, context), { clientOnly: clientOnly, deferred: deferred }));
    }
    return flavored;
}
var StoreWriter = (function () {
    function StoreWriter(cache, reader, fragments) {
        this.cache = cache;
        this.reader = reader;
        this.fragments = fragments;
    }
    StoreWriter.prototype.writeToStore = function (store, _a) {
        var _this = this;
        var query = _a.query, result = _a.result, dataId = _a.dataId, variables = _a.variables, overwrite = _a.overwrite;
        var operationDefinition = getOperationDefinition(query);
        var merger = makeProcessedFieldsMerger();
        variables = __assign$2(__assign$2({}, getDefaultValues(operationDefinition)), variables);
        var context = __assign$2(__assign$2({ store: store, written: Object.create(null), merge: function (existing, incoming) {
                return merger.merge(existing, incoming);
            }, variables: variables, varString: canonicalStringify(variables) }, extractFragmentContext(query, this.fragments)), { overwrite: !!overwrite, incomingById: new Map, clientOnly: false, deferred: false, flavors: new Map });
        var ref = this.processSelectionSet({
            result: result || Object.create(null),
            dataId: dataId,
            selectionSet: operationDefinition.selectionSet,
            mergeTree: { map: new Map },
            context: context,
        });
        if (!isReference(ref)) {
            throw __DEV__ ? new InvariantError("Could not identify object ".concat(JSON.stringify(result))) : new InvariantError(7);
        }
        context.incomingById.forEach(function (_a, dataId) {
            var storeObject = _a.storeObject, mergeTree = _a.mergeTree, fieldNodeSet = _a.fieldNodeSet;
            var entityRef = makeReference(dataId);
            if (mergeTree && mergeTree.map.size) {
                var applied = _this.applyMerges(mergeTree, entityRef, storeObject, context);
                if (isReference(applied)) {
                    return;
                }
                storeObject = applied;
            }
            if (__DEV__ && !context.overwrite) {
                var fieldsWithSelectionSets_1 = Object.create(null);
                fieldNodeSet.forEach(function (field) {
                    if (field.selectionSet) {
                        fieldsWithSelectionSets_1[field.name.value] = true;
                    }
                });
                var hasSelectionSet_1 = function (storeFieldName) {
                    return fieldsWithSelectionSets_1[fieldNameFromStoreName(storeFieldName)] === true;
                };
                var hasMergeFunction_1 = function (storeFieldName) {
                    var childTree = mergeTree && mergeTree.map.get(storeFieldName);
                    return Boolean(childTree && childTree.info && childTree.info.merge);
                };
                Object.keys(storeObject).forEach(function (storeFieldName) {
                    if (hasSelectionSet_1(storeFieldName) &&
                        !hasMergeFunction_1(storeFieldName)) {
                        warnAboutDataLoss(entityRef, storeObject, storeFieldName, context.store);
                    }
                });
            }
            store.merge(dataId, storeObject);
        });
        store.retain(ref.__ref);
        return ref;
    };
    StoreWriter.prototype.processSelectionSet = function (_a) {
        var _this = this;
        var dataId = _a.dataId, result = _a.result, selectionSet = _a.selectionSet, context = _a.context, mergeTree = _a.mergeTree;
        var policies = this.cache.policies;
        var incoming = Object.create(null);
        var typename = (dataId && policies.rootTypenamesById[dataId]) ||
            getTypenameFromResult(result, selectionSet, context.fragmentMap) ||
            (dataId && context.store.get(dataId, "__typename"));
        if ("string" === typeof typename) {
            incoming.__typename = typename;
        }
        var readField = function () {
            var options = normalizeReadFieldOptions(arguments, incoming, context.variables);
            if (isReference(options.from)) {
                var info = context.incomingById.get(options.from.__ref);
                if (info) {
                    var result_1 = policies.readField(__assign$2(__assign$2({}, options), { from: info.storeObject }), context);
                    if (result_1 !== void 0) {
                        return result_1;
                    }
                }
            }
            return policies.readField(options, context);
        };
        var fieldNodeSet = new Set();
        this.flattenFields(selectionSet, result, context, typename).forEach(function (context, field) {
            var _a;
            var resultFieldKey = resultKeyNameFromField(field);
            var value = result[resultFieldKey];
            fieldNodeSet.add(field);
            if (value !== void 0) {
                var storeFieldName = policies.getStoreFieldName({
                    typename: typename,
                    fieldName: field.name.value,
                    field: field,
                    variables: context.variables,
                });
                var childTree = getChildMergeTree(mergeTree, storeFieldName);
                var incomingValue = _this.processFieldValue(value, field, field.selectionSet
                    ? getContextFlavor(context, false, false)
                    : context, childTree);
                var childTypename = void 0;
                if (field.selectionSet &&
                    (isReference(incomingValue) ||
                        storeValueIsStoreObject(incomingValue))) {
                    childTypename = readField("__typename", incomingValue);
                }
                var merge = policies.getMergeFunction(typename, field.name.value, childTypename);
                if (merge) {
                    childTree.info = {
                        field: field,
                        typename: typename,
                        merge: merge,
                    };
                }
                else {
                    maybeRecycleChildMergeTree(mergeTree, storeFieldName);
                }
                incoming = context.merge(incoming, (_a = {},
                    _a[storeFieldName] = incomingValue,
                    _a));
            }
            else if (__DEV__ &&
                !context.clientOnly &&
                !context.deferred &&
                !addTypenameToDocument.added(field) &&
                !policies.getReadFunction(typename, field.name.value)) {
                __DEV__ && invariant$2.error("Missing field '".concat(resultKeyNameFromField(field), "' while writing result ").concat(JSON.stringify(result, null, 2)).substring(0, 1000));
            }
        });
        try {
            var _b = policies.identify(result, {
                typename: typename,
                selectionSet: selectionSet,
                fragmentMap: context.fragmentMap,
                storeObject: incoming,
                readField: readField,
            }), id = _b[0], keyObject = _b[1];
            dataId = dataId || id;
            if (keyObject) {
                incoming = context.merge(incoming, keyObject);
            }
        }
        catch (e) {
            if (!dataId)
                throw e;
        }
        if ("string" === typeof dataId) {
            var dataRef = makeReference(dataId);
            var sets = context.written[dataId] || (context.written[dataId] = []);
            if (sets.indexOf(selectionSet) >= 0)
                return dataRef;
            sets.push(selectionSet);
            if (this.reader && this.reader.isFresh(result, dataRef, selectionSet, context)) {
                return dataRef;
            }
            var previous_1 = context.incomingById.get(dataId);
            if (previous_1) {
                previous_1.storeObject = context.merge(previous_1.storeObject, incoming);
                previous_1.mergeTree = mergeMergeTrees(previous_1.mergeTree, mergeTree);
                fieldNodeSet.forEach(function (field) { return previous_1.fieldNodeSet.add(field); });
            }
            else {
                context.incomingById.set(dataId, {
                    storeObject: incoming,
                    mergeTree: mergeTreeIsEmpty(mergeTree) ? void 0 : mergeTree,
                    fieldNodeSet: fieldNodeSet,
                });
            }
            return dataRef;
        }
        return incoming;
    };
    StoreWriter.prototype.processFieldValue = function (value, field, context, mergeTree) {
        var _this = this;
        if (!field.selectionSet || value === null) {
            return __DEV__ ? cloneDeep(value) : value;
        }
        if (isArray(value)) {
            return value.map(function (item, i) {
                var value = _this.processFieldValue(item, field, context, getChildMergeTree(mergeTree, i));
                maybeRecycleChildMergeTree(mergeTree, i);
                return value;
            });
        }
        return this.processSelectionSet({
            result: value,
            selectionSet: field.selectionSet,
            context: context,
            mergeTree: mergeTree,
        });
    };
    StoreWriter.prototype.flattenFields = function (selectionSet, result, context, typename) {
        if (typename === void 0) { typename = getTypenameFromResult(result, selectionSet, context.fragmentMap); }
        var fieldMap = new Map();
        var policies = this.cache.policies;
        var limitingTrie = new Trie(false);
        (function flatten(selectionSet, inheritedContext) {
            var visitedNode = limitingTrie.lookup(selectionSet, inheritedContext.clientOnly, inheritedContext.deferred);
            if (visitedNode.visited)
                return;
            visitedNode.visited = true;
            selectionSet.selections.forEach(function (selection) {
                if (!shouldInclude(selection, context.variables))
                    return;
                var clientOnly = inheritedContext.clientOnly, deferred = inheritedContext.deferred;
                if (!(clientOnly && deferred) &&
                    isNonEmptyArray(selection.directives)) {
                    selection.directives.forEach(function (dir) {
                        var name = dir.name.value;
                        if (name === "client")
                            clientOnly = true;
                        if (name === "defer") {
                            var args = argumentsObjectFromField(dir, context.variables);
                            if (!args || args.if !== false) {
                                deferred = true;
                            }
                        }
                    });
                }
                if (isField(selection)) {
                    var existing = fieldMap.get(selection);
                    if (existing) {
                        clientOnly = clientOnly && existing.clientOnly;
                        deferred = deferred && existing.deferred;
                    }
                    fieldMap.set(selection, getContextFlavor(context, clientOnly, deferred));
                }
                else {
                    var fragment = getFragmentFromSelection(selection, context.lookupFragment);
                    if (!fragment && selection.kind === Kind$1.FRAGMENT_SPREAD) {
                        throw __DEV__ ? new InvariantError("No fragment named ".concat(selection.name.value)) : new InvariantError(8);
                    }
                    if (fragment &&
                        policies.fragmentMatches(fragment, typename, result, context.variables)) {
                        flatten(fragment.selectionSet, getContextFlavor(context, clientOnly, deferred));
                    }
                }
            });
        })(selectionSet, context);
        return fieldMap;
    };
    StoreWriter.prototype.applyMerges = function (mergeTree, existing, incoming, context, getStorageArgs) {
        var _a;
        var _this = this;
        if (mergeTree.map.size && !isReference(incoming)) {
            var e_1 = (!isArray(incoming) &&
                (isReference(existing) || storeValueIsStoreObject(existing))) ? existing : void 0;
            var i_1 = incoming;
            if (e_1 && !getStorageArgs) {
                getStorageArgs = [isReference(e_1) ? e_1.__ref : e_1];
            }
            var changedFields_1;
            var getValue_1 = function (from, name) {
                return isArray(from)
                    ? (typeof name === "number" ? from[name] : void 0)
                    : context.store.getFieldValue(from, String(name));
            };
            mergeTree.map.forEach(function (childTree, storeFieldName) {
                var eVal = getValue_1(e_1, storeFieldName);
                var iVal = getValue_1(i_1, storeFieldName);
                if (void 0 === iVal)
                    return;
                if (getStorageArgs) {
                    getStorageArgs.push(storeFieldName);
                }
                var aVal = _this.applyMerges(childTree, eVal, iVal, context, getStorageArgs);
                if (aVal !== iVal) {
                    changedFields_1 = changedFields_1 || new Map;
                    changedFields_1.set(storeFieldName, aVal);
                }
                if (getStorageArgs) {
                    invariant$2(getStorageArgs.pop() === storeFieldName);
                }
            });
            if (changedFields_1) {
                incoming = (isArray(i_1) ? i_1.slice(0) : __assign$2({}, i_1));
                changedFields_1.forEach(function (value, name) {
                    incoming[name] = value;
                });
            }
        }
        if (mergeTree.info) {
            return this.cache.policies.runMergeFunction(existing, incoming, mergeTree.info, context, getStorageArgs && (_a = context.store).getStorage.apply(_a, getStorageArgs));
        }
        return incoming;
    };
    return StoreWriter;
}());
var emptyMergeTreePool = [];
function getChildMergeTree(_a, name) {
    var map = _a.map;
    if (!map.has(name)) {
        map.set(name, emptyMergeTreePool.pop() || { map: new Map });
    }
    return map.get(name);
}
function mergeMergeTrees(left, right) {
    if (left === right || !right || mergeTreeIsEmpty(right))
        return left;
    if (!left || mergeTreeIsEmpty(left))
        return right;
    var info = left.info && right.info ? __assign$2(__assign$2({}, left.info), right.info) : left.info || right.info;
    var needToMergeMaps = left.map.size && right.map.size;
    var map = needToMergeMaps ? new Map :
        left.map.size ? left.map : right.map;
    var merged = { info: info, map: map };
    if (needToMergeMaps) {
        var remainingRightKeys_1 = new Set(right.map.keys());
        left.map.forEach(function (leftTree, key) {
            merged.map.set(key, mergeMergeTrees(leftTree, right.map.get(key)));
            remainingRightKeys_1.delete(key);
        });
        remainingRightKeys_1.forEach(function (key) {
            merged.map.set(key, mergeMergeTrees(right.map.get(key), left.map.get(key)));
        });
    }
    return merged;
}
function mergeTreeIsEmpty(tree) {
    return !tree || !(tree.info || tree.map.size);
}
function maybeRecycleChildMergeTree(_a, name) {
    var map = _a.map;
    var childTree = map.get(name);
    if (childTree && mergeTreeIsEmpty(childTree)) {
        emptyMergeTreePool.push(childTree);
        map.delete(name);
    }
}
var warnings = new Set();
function warnAboutDataLoss(existingRef, incomingObj, storeFieldName, store) {
    var getChild = function (objOrRef) {
        var child = store.getFieldValue(objOrRef, storeFieldName);
        return typeof child === "object" && child;
    };
    var existing = getChild(existingRef);
    if (!existing)
        return;
    var incoming = getChild(incomingObj);
    if (!incoming)
        return;
    if (isReference(existing))
        return;
    if (equal(existing, incoming))
        return;
    if (Object.keys(existing).every(function (key) { return store.getFieldValue(incoming, key) !== void 0; })) {
        return;
    }
    var parentType = store.getFieldValue(existingRef, "__typename") ||
        store.getFieldValue(incomingObj, "__typename");
    var fieldName = fieldNameFromStoreName(storeFieldName);
    var typeDotName = "".concat(parentType, ".").concat(fieldName);
    if (warnings.has(typeDotName))
        return;
    warnings.add(typeDotName);
    var childTypenames = [];
    if (!isArray(existing) &&
        !isArray(incoming)) {
        [existing, incoming].forEach(function (child) {
            var typename = store.getFieldValue(child, "__typename");
            if (typeof typename === "string" &&
                !childTypenames.includes(typename)) {
                childTypenames.push(typename);
            }
        });
    }
    __DEV__ && invariant$2.warn("Cache data may be lost when replacing the ".concat(fieldName, " field of a ").concat(parentType, " object.\n\nTo address this problem (which is not a bug in Apollo Client), ").concat(childTypenames.length
        ? "either ensure all objects of type " +
            childTypenames.join(" and ") + " have an ID or a custom merge function, or "
        : "", "define a custom merge function for the ").concat(typeDotName, " field, so InMemoryCache can safely merge these objects:\n\n  existing: ").concat(JSON.stringify(existing).slice(0, 1000), "\n  incoming: ").concat(JSON.stringify(incoming).slice(0, 1000), "\n\nFor more information about these options, please refer to the documentation:\n\n  * Ensuring entity objects have IDs: https://go.apollo.dev/c/generating-unique-identifiers\n  * Defining custom merge functions: https://go.apollo.dev/c/merging-non-normalized-objects\n"));
}

var InMemoryCache = (function (_super) {
    __extends(InMemoryCache, _super);
    function InMemoryCache(config) {
        if (config === void 0) { config = {}; }
        var _this = _super.call(this) || this;
        _this.watches = new Set();
        _this.typenameDocumentCache = new Map();
        _this.makeVar = makeVar;
        _this.txCount = 0;
        _this.config = normalizeConfig(config);
        _this.addTypename = !!_this.config.addTypename;
        _this.policies = new Policies({
            cache: _this,
            dataIdFromObject: _this.config.dataIdFromObject,
            possibleTypes: _this.config.possibleTypes,
            typePolicies: _this.config.typePolicies,
        });
        _this.init();
        return _this;
    }
    InMemoryCache.prototype.init = function () {
        var rootStore = this.data = new EntityStore.Root({
            policies: this.policies,
            resultCaching: this.config.resultCaching,
        });
        this.optimisticData = rootStore.stump;
        this.resetResultCache();
    };
    InMemoryCache.prototype.resetResultCache = function (resetResultIdentities) {
        var _this = this;
        var previousReader = this.storeReader;
        var fragments = this.config.fragments;
        this.storeWriter = new StoreWriter(this, this.storeReader = new StoreReader({
            cache: this,
            addTypename: this.addTypename,
            resultCacheMaxSize: this.config.resultCacheMaxSize,
            canonizeResults: shouldCanonizeResults(this.config),
            canon: resetResultIdentities
                ? void 0
                : previousReader && previousReader.canon,
            fragments: fragments,
        }), fragments);
        this.maybeBroadcastWatch = wrap(function (c, options) {
            return _this.broadcastWatch(c, options);
        }, {
            max: this.config.resultCacheMaxSize,
            makeCacheKey: function (c) {
                var store = c.optimistic ? _this.optimisticData : _this.data;
                if (supportsResultCaching(store)) {
                    var optimistic = c.optimistic, id = c.id, variables = c.variables;
                    return store.makeCacheKey(c.query, c.callback, canonicalStringify({ optimistic: optimistic, id: id, variables: variables }));
                }
            }
        });
        new Set([
            this.data.group,
            this.optimisticData.group,
        ]).forEach(function (group) { return group.resetCaching(); });
    };
    InMemoryCache.prototype.restore = function (data) {
        this.init();
        if (data)
            this.data.replace(data);
        return this;
    };
    InMemoryCache.prototype.extract = function (optimistic) {
        if (optimistic === void 0) { optimistic = false; }
        return (optimistic ? this.optimisticData : this.data).extract();
    };
    InMemoryCache.prototype.read = function (options) {
        var _a = options.returnPartialData, returnPartialData = _a === void 0 ? false : _a;
        try {
            return this.storeReader.diffQueryAgainstStore(__assign$2(__assign$2({}, options), { store: options.optimistic ? this.optimisticData : this.data, config: this.config, returnPartialData: returnPartialData })).result || null;
        }
        catch (e) {
            if (e instanceof MissingFieldError) {
                return null;
            }
            throw e;
        }
    };
    InMemoryCache.prototype.write = function (options) {
        try {
            ++this.txCount;
            return this.storeWriter.writeToStore(this.data, options);
        }
        finally {
            if (!--this.txCount && options.broadcast !== false) {
                this.broadcastWatches();
            }
        }
    };
    InMemoryCache.prototype.modify = function (options) {
        if (hasOwn.call(options, "id") && !options.id) {
            return false;
        }
        var store = options.optimistic
            ? this.optimisticData
            : this.data;
        try {
            ++this.txCount;
            return store.modify(options.id || "ROOT_QUERY", options.fields);
        }
        finally {
            if (!--this.txCount && options.broadcast !== false) {
                this.broadcastWatches();
            }
        }
    };
    InMemoryCache.prototype.diff = function (options) {
        return this.storeReader.diffQueryAgainstStore(__assign$2(__assign$2({}, options), { store: options.optimistic ? this.optimisticData : this.data, rootId: options.id || "ROOT_QUERY", config: this.config }));
    };
    InMemoryCache.prototype.watch = function (watch) {
        var _this = this;
        if (!this.watches.size) {
            recallCache(this);
        }
        this.watches.add(watch);
        if (watch.immediate) {
            this.maybeBroadcastWatch(watch);
        }
        return function () {
            if (_this.watches.delete(watch) && !_this.watches.size) {
                forgetCache(_this);
            }
            _this.maybeBroadcastWatch.forget(watch);
        };
    };
    InMemoryCache.prototype.gc = function (options) {
        canonicalStringify.reset();
        var ids = this.optimisticData.gc();
        if (options && !this.txCount) {
            if (options.resetResultCache) {
                this.resetResultCache(options.resetResultIdentities);
            }
            else if (options.resetResultIdentities) {
                this.storeReader.resetCanon();
            }
        }
        return ids;
    };
    InMemoryCache.prototype.retain = function (rootId, optimistic) {
        return (optimistic ? this.optimisticData : this.data).retain(rootId);
    };
    InMemoryCache.prototype.release = function (rootId, optimistic) {
        return (optimistic ? this.optimisticData : this.data).release(rootId);
    };
    InMemoryCache.prototype.identify = function (object) {
        if (isReference(object))
            return object.__ref;
        try {
            return this.policies.identify(object)[0];
        }
        catch (e) {
            __DEV__ && invariant$2.warn(e);
        }
    };
    InMemoryCache.prototype.evict = function (options) {
        if (!options.id) {
            if (hasOwn.call(options, "id")) {
                return false;
            }
            options = __assign$2(__assign$2({}, options), { id: "ROOT_QUERY" });
        }
        try {
            ++this.txCount;
            return this.optimisticData.evict(options, this.data);
        }
        finally {
            if (!--this.txCount && options.broadcast !== false) {
                this.broadcastWatches();
            }
        }
    };
    InMemoryCache.prototype.reset = function (options) {
        var _this = this;
        this.init();
        canonicalStringify.reset();
        if (options && options.discardWatches) {
            this.watches.forEach(function (watch) { return _this.maybeBroadcastWatch.forget(watch); });
            this.watches.clear();
            forgetCache(this);
        }
        else {
            this.broadcastWatches();
        }
        return Promise.resolve();
    };
    InMemoryCache.prototype.removeOptimistic = function (idToRemove) {
        var newOptimisticData = this.optimisticData.removeLayer(idToRemove);
        if (newOptimisticData !== this.optimisticData) {
            this.optimisticData = newOptimisticData;
            this.broadcastWatches();
        }
    };
    InMemoryCache.prototype.batch = function (options) {
        var _this = this;
        var update = options.update, _a = options.optimistic, optimistic = _a === void 0 ? true : _a, removeOptimistic = options.removeOptimistic, onWatchUpdated = options.onWatchUpdated;
        var updateResult;
        var perform = function (layer) {
            var _a = _this, data = _a.data, optimisticData = _a.optimisticData;
            ++_this.txCount;
            if (layer) {
                _this.data = _this.optimisticData = layer;
            }
            try {
                return updateResult = update(_this);
            }
            finally {
                --_this.txCount;
                _this.data = data;
                _this.optimisticData = optimisticData;
            }
        };
        var alreadyDirty = new Set();
        if (onWatchUpdated && !this.txCount) {
            this.broadcastWatches(__assign$2(__assign$2({}, options), { onWatchUpdated: function (watch) {
                    alreadyDirty.add(watch);
                    return false;
                } }));
        }
        if (typeof optimistic === 'string') {
            this.optimisticData = this.optimisticData.addLayer(optimistic, perform);
        }
        else if (optimistic === false) {
            perform(this.data);
        }
        else {
            perform();
        }
        if (typeof removeOptimistic === "string") {
            this.optimisticData = this.optimisticData.removeLayer(removeOptimistic);
        }
        if (onWatchUpdated && alreadyDirty.size) {
            this.broadcastWatches(__assign$2(__assign$2({}, options), { onWatchUpdated: function (watch, diff) {
                    var result = onWatchUpdated.call(this, watch, diff);
                    if (result !== false) {
                        alreadyDirty.delete(watch);
                    }
                    return result;
                } }));
            if (alreadyDirty.size) {
                alreadyDirty.forEach(function (watch) { return _this.maybeBroadcastWatch.dirty(watch); });
            }
        }
        else {
            this.broadcastWatches(options);
        }
        return updateResult;
    };
    InMemoryCache.prototype.performTransaction = function (update, optimisticId) {
        return this.batch({
            update: update,
            optimistic: optimisticId || (optimisticId !== null),
        });
    };
    InMemoryCache.prototype.transformDocument = function (document) {
        if (this.addTypename) {
            var result = this.typenameDocumentCache.get(document);
            if (!result) {
                result = addTypenameToDocument(document);
                this.typenameDocumentCache.set(document, result);
                this.typenameDocumentCache.set(result, result);
            }
            return result;
        }
        return document;
    };
    InMemoryCache.prototype.transformForLink = function (document) {
        var fragments = this.config.fragments;
        return fragments
            ? fragments.transform(document)
            : document;
    };
    InMemoryCache.prototype.broadcastWatches = function (options) {
        var _this = this;
        if (!this.txCount) {
            this.watches.forEach(function (c) { return _this.maybeBroadcastWatch(c, options); });
        }
    };
    InMemoryCache.prototype.broadcastWatch = function (c, options) {
        var lastDiff = c.lastDiff;
        var diff = this.diff(c);
        if (options) {
            if (c.optimistic &&
                typeof options.optimistic === "string") {
                diff.fromOptimisticTransaction = true;
            }
            if (options.onWatchUpdated &&
                options.onWatchUpdated.call(this, c, diff, lastDiff) === false) {
                return;
            }
        }
        if (!lastDiff || !equal(lastDiff.result, diff.result)) {
            c.callback(c.lastDiff = diff, lastDiff);
        }
    };
    return InMemoryCache;
}(ApolloCache));

function isApolloError(err) {
    return err.hasOwnProperty('graphQLErrors');
}
var generateErrorMessage = function (err) {
    var message = '';
    if (isNonEmptyArray(err.graphQLErrors) || isNonEmptyArray(err.clientErrors)) {
        var errors = (err.graphQLErrors || [])
            .concat(err.clientErrors || []);
        errors.forEach(function (error) {
            var errorMessage = error
                ? error.message
                : 'Error message not found.';
            message += "".concat(errorMessage, "\n");
        });
    }
    if (err.networkError) {
        message += "".concat(err.networkError.message, "\n");
    }
    message = message.replace(/\n$/, '');
    return message;
};
var ApolloError = (function (_super) {
    __extends(ApolloError, _super);
    function ApolloError(_a) {
        var graphQLErrors = _a.graphQLErrors, clientErrors = _a.clientErrors, networkError = _a.networkError, errorMessage = _a.errorMessage, extraInfo = _a.extraInfo;
        var _this = _super.call(this, errorMessage) || this;
        _this.name = 'ApolloError';
        _this.graphQLErrors = graphQLErrors || [];
        _this.clientErrors = clientErrors || [];
        _this.networkError = networkError || null;
        _this.message = errorMessage || generateErrorMessage(_this);
        _this.extraInfo = extraInfo;
        _this.__proto__ = ApolloError.prototype;
        return _this;
    }
    return ApolloError;
}(Error));

var NetworkStatus;
(function (NetworkStatus) {
    NetworkStatus[NetworkStatus["loading"] = 1] = "loading";
    NetworkStatus[NetworkStatus["setVariables"] = 2] = "setVariables";
    NetworkStatus[NetworkStatus["fetchMore"] = 3] = "fetchMore";
    NetworkStatus[NetworkStatus["refetch"] = 4] = "refetch";
    NetworkStatus[NetworkStatus["poll"] = 6] = "poll";
    NetworkStatus[NetworkStatus["ready"] = 7] = "ready";
    NetworkStatus[NetworkStatus["error"] = 8] = "error";
})(NetworkStatus || (NetworkStatus = {}));
function isNetworkRequestInFlight(networkStatus) {
    return networkStatus ? networkStatus < 7 : false;
}

var assign = Object.assign, hasOwnProperty$1 = Object.hasOwnProperty;
var ObservableQuery = (function (_super) {
    __extends(ObservableQuery, _super);
    function ObservableQuery(_a) {
        var queryManager = _a.queryManager, queryInfo = _a.queryInfo, options = _a.options;
        var _this = _super.call(this, function (observer) {
            try {
                var subObserver = observer._subscription._observer;
                if (subObserver && !subObserver.error) {
                    subObserver.error = defaultSubscriptionObserverErrorCallback;
                }
            }
            catch (_a) { }
            var first = !_this.observers.size;
            _this.observers.add(observer);
            var last = _this.last;
            if (last && last.error) {
                observer.error && observer.error(last.error);
            }
            else if (last && last.result) {
                observer.next && observer.next(last.result);
            }
            if (first) {
                _this.reobserve().catch(function () { });
            }
            return function () {
                if (_this.observers.delete(observer) && !_this.observers.size) {
                    _this.tearDownQuery();
                }
            };
        }) || this;
        _this.observers = new Set();
        _this.subscriptions = new Set();
        _this.queryInfo = queryInfo;
        _this.queryManager = queryManager;
        _this.isTornDown = false;
        var _b = queryManager.defaultOptions.watchQuery, _c = _b === void 0 ? {} : _b, _d = _c.fetchPolicy, defaultFetchPolicy = _d === void 0 ? "cache-first" : _d;
        var _e = options.fetchPolicy, fetchPolicy = _e === void 0 ? defaultFetchPolicy : _e, _f = options.initialFetchPolicy, initialFetchPolicy = _f === void 0 ? (fetchPolicy === "standby" ? defaultFetchPolicy : fetchPolicy) : _f;
        _this.options = __assign$2(__assign$2({}, options), { initialFetchPolicy: initialFetchPolicy, fetchPolicy: fetchPolicy });
        _this.queryId = queryInfo.queryId || queryManager.generateQueryId();
        var opDef = getOperationDefinition(_this.query);
        _this.queryName = opDef && opDef.name && opDef.name.value;
        return _this;
    }
    Object.defineProperty(ObservableQuery.prototype, "query", {
        get: function () {
            return this.queryManager.transform(this.options.query).document;
        },
        enumerable: false,
        configurable: true
    });
    Object.defineProperty(ObservableQuery.prototype, "variables", {
        get: function () {
            return this.options.variables;
        },
        enumerable: false,
        configurable: true
    });
    ObservableQuery.prototype.result = function () {
        var _this = this;
        return new Promise(function (resolve, reject) {
            var observer = {
                next: function (result) {
                    resolve(result);
                    _this.observers.delete(observer);
                    if (!_this.observers.size) {
                        _this.queryManager.removeQuery(_this.queryId);
                    }
                    setTimeout(function () {
                        subscription.unsubscribe();
                    }, 0);
                },
                error: reject,
            };
            var subscription = _this.subscribe(observer);
        });
    };
    ObservableQuery.prototype.getCurrentResult = function (saveAsLastResult) {
        if (saveAsLastResult === void 0) { saveAsLastResult = true; }
        var lastResult = this.getLastResult(true);
        var networkStatus = this.queryInfo.networkStatus ||
            (lastResult && lastResult.networkStatus) ||
            NetworkStatus.ready;
        var result = __assign$2(__assign$2({}, lastResult), { loading: isNetworkRequestInFlight(networkStatus), networkStatus: networkStatus });
        var _a = this.options.fetchPolicy, fetchPolicy = _a === void 0 ? "cache-first" : _a;
        if (fetchPolicy === 'network-only' ||
            fetchPolicy === 'no-cache' ||
            fetchPolicy === 'standby' ||
            this.queryManager.transform(this.options.query).hasForcedResolvers) ;
        else {
            var diff = this.queryInfo.getDiff();
            if (diff.complete || this.options.returnPartialData) {
                result.data = diff.result;
            }
            if (equal(result.data, {})) {
                result.data = void 0;
            }
            if (diff.complete) {
                delete result.partial;
                if (diff.complete &&
                    result.networkStatus === NetworkStatus.loading &&
                    (fetchPolicy === 'cache-first' ||
                        fetchPolicy === 'cache-only')) {
                    result.networkStatus = NetworkStatus.ready;
                    result.loading = false;
                }
            }
            else {
                result.partial = true;
            }
            if (__DEV__ &&
                !diff.complete &&
                !this.options.partialRefetch &&
                !result.loading &&
                !result.data &&
                !result.error) {
                logMissingFieldErrors(diff.missing);
            }
        }
        if (saveAsLastResult) {
            this.updateLastResult(result);
        }
        return result;
    };
    ObservableQuery.prototype.isDifferentFromLastResult = function (newResult, variables) {
        return (!this.last ||
            !equal(this.last.result, newResult) ||
            (variables && !equal(this.last.variables, variables)));
    };
    ObservableQuery.prototype.getLast = function (key, variablesMustMatch) {
        var last = this.last;
        if (last &&
            last[key] &&
            (!variablesMustMatch || equal(last.variables, this.variables))) {
            return last[key];
        }
    };
    ObservableQuery.prototype.getLastResult = function (variablesMustMatch) {
        return this.getLast("result", variablesMustMatch);
    };
    ObservableQuery.prototype.getLastError = function (variablesMustMatch) {
        return this.getLast("error", variablesMustMatch);
    };
    ObservableQuery.prototype.resetLastResults = function () {
        delete this.last;
        this.isTornDown = false;
    };
    ObservableQuery.prototype.resetQueryStoreErrors = function () {
        this.queryManager.resetErrors(this.queryId);
    };
    ObservableQuery.prototype.refetch = function (variables) {
        var _a;
        var reobserveOptions = {
            pollInterval: 0,
        };
        var fetchPolicy = this.options.fetchPolicy;
        if (fetchPolicy === 'cache-and-network') {
            reobserveOptions.fetchPolicy = fetchPolicy;
        }
        else if (fetchPolicy === 'no-cache') {
            reobserveOptions.fetchPolicy = 'no-cache';
        }
        else {
            reobserveOptions.fetchPolicy = 'network-only';
        }
        if (__DEV__ && variables && hasOwnProperty$1.call(variables, "variables")) {
            var queryDef = getQueryDefinition(this.query);
            var vars = queryDef.variableDefinitions;
            if (!vars || !vars.some(function (v) { return v.variable.name.value === "variables"; })) {
                __DEV__ && invariant$2.warn("Called refetch(".concat(JSON.stringify(variables), ") for query ").concat(((_a = queryDef.name) === null || _a === void 0 ? void 0 : _a.value) || JSON.stringify(queryDef), ", which does not declare a $variables variable.\nDid you mean to call refetch(variables) instead of refetch({ variables })?"));
            }
        }
        if (variables && !equal(this.options.variables, variables)) {
            reobserveOptions.variables = this.options.variables = __assign$2(__assign$2({}, this.options.variables), variables);
        }
        this.queryInfo.resetLastWrite();
        return this.reobserve(reobserveOptions, NetworkStatus.refetch);
    };
    ObservableQuery.prototype.fetchMore = function (fetchMoreOptions) {
        var _this = this;
        var combinedOptions = __assign$2(__assign$2({}, (fetchMoreOptions.query ? fetchMoreOptions : __assign$2(__assign$2(__assign$2(__assign$2({}, this.options), { query: this.query }), fetchMoreOptions), { variables: __assign$2(__assign$2({}, this.options.variables), fetchMoreOptions.variables) }))), { fetchPolicy: "no-cache" });
        var qid = this.queryManager.generateQueryId();
        var queryInfo = this.queryInfo;
        var originalNetworkStatus = queryInfo.networkStatus;
        queryInfo.networkStatus = NetworkStatus.fetchMore;
        if (combinedOptions.notifyOnNetworkStatusChange) {
            this.observe();
        }
        var updatedQuerySet = new Set();
        return this.queryManager.fetchQuery(qid, combinedOptions, NetworkStatus.fetchMore).then(function (fetchMoreResult) {
            _this.queryManager.removeQuery(qid);
            if (queryInfo.networkStatus === NetworkStatus.fetchMore) {
                queryInfo.networkStatus = originalNetworkStatus;
            }
            _this.queryManager.cache.batch({
                update: function (cache) {
                    var updateQuery = fetchMoreOptions.updateQuery;
                    if (updateQuery) {
                        cache.updateQuery({
                            query: _this.query,
                            variables: _this.variables,
                            returnPartialData: true,
                            optimistic: false,
                        }, function (previous) { return updateQuery(previous, {
                            fetchMoreResult: fetchMoreResult.data,
                            variables: combinedOptions.variables,
                        }); });
                    }
                    else {
                        cache.writeQuery({
                            query: combinedOptions.query,
                            variables: combinedOptions.variables,
                            data: fetchMoreResult.data,
                        });
                    }
                },
                onWatchUpdated: function (watch) {
                    updatedQuerySet.add(watch.query);
                },
            });
            return fetchMoreResult;
        }).finally(function () {
            if (!updatedQuerySet.has(_this.query)) {
                reobserveCacheFirst(_this);
            }
        });
    };
    ObservableQuery.prototype.subscribeToMore = function (options) {
        var _this = this;
        var subscription = this.queryManager
            .startGraphQLSubscription({
            query: options.document,
            variables: options.variables,
            context: options.context,
        })
            .subscribe({
            next: function (subscriptionData) {
                var updateQuery = options.updateQuery;
                if (updateQuery) {
                    _this.updateQuery(function (previous, _a) {
                        var variables = _a.variables;
                        return updateQuery(previous, {
                            subscriptionData: subscriptionData,
                            variables: variables,
                        });
                    });
                }
            },
            error: function (err) {
                if (options.onError) {
                    options.onError(err);
                    return;
                }
                __DEV__ && invariant$2.error('Unhandled GraphQL subscription error', err);
            },
        });
        this.subscriptions.add(subscription);
        return function () {
            if (_this.subscriptions.delete(subscription)) {
                subscription.unsubscribe();
            }
        };
    };
    ObservableQuery.prototype.setOptions = function (newOptions) {
        return this.reobserve(newOptions);
    };
    ObservableQuery.prototype.setVariables = function (variables) {
        if (equal(this.variables, variables)) {
            return this.observers.size
                ? this.result()
                : Promise.resolve();
        }
        this.options.variables = variables;
        if (!this.observers.size) {
            return Promise.resolve();
        }
        return this.reobserve({
            fetchPolicy: this.options.initialFetchPolicy,
            variables: variables,
        }, NetworkStatus.setVariables);
    };
    ObservableQuery.prototype.updateQuery = function (mapFn) {
        var queryManager = this.queryManager;
        var result = queryManager.cache.diff({
            query: this.options.query,
            variables: this.variables,
            returnPartialData: true,
            optimistic: false,
        }).result;
        var newResult = mapFn(result, {
            variables: this.variables,
        });
        if (newResult) {
            queryManager.cache.writeQuery({
                query: this.options.query,
                data: newResult,
                variables: this.variables,
            });
            queryManager.broadcastQueries();
        }
    };
    ObservableQuery.prototype.startPolling = function (pollInterval) {
        this.options.pollInterval = pollInterval;
        this.updatePolling();
    };
    ObservableQuery.prototype.stopPolling = function () {
        this.options.pollInterval = 0;
        this.updatePolling();
    };
    ObservableQuery.prototype.applyNextFetchPolicy = function (reason, options) {
        if (options.nextFetchPolicy) {
            var _a = options.fetchPolicy, fetchPolicy = _a === void 0 ? "cache-first" : _a, _b = options.initialFetchPolicy, initialFetchPolicy = _b === void 0 ? fetchPolicy : _b;
            if (fetchPolicy === "standby") ;
            else if (typeof options.nextFetchPolicy === "function") {
                options.fetchPolicy = options.nextFetchPolicy(fetchPolicy, {
                    reason: reason,
                    options: options,
                    observable: this,
                    initialFetchPolicy: initialFetchPolicy,
                });
            }
            else if (reason === "variables-changed") {
                options.fetchPolicy = initialFetchPolicy;
            }
            else {
                options.fetchPolicy = options.nextFetchPolicy;
            }
        }
        return options.fetchPolicy;
    };
    ObservableQuery.prototype.fetch = function (options, newNetworkStatus) {
        this.queryManager.setObservableQuery(this);
        return this.queryManager.fetchQueryObservable(this.queryId, options, newNetworkStatus);
    };
    ObservableQuery.prototype.updatePolling = function () {
        var _this = this;
        if (this.queryManager.ssrMode) {
            return;
        }
        var _a = this, pollingInfo = _a.pollingInfo, pollInterval = _a.options.pollInterval;
        if (!pollInterval) {
            if (pollingInfo) {
                clearTimeout(pollingInfo.timeout);
                delete this.pollingInfo;
            }
            return;
        }
        if (pollingInfo &&
            pollingInfo.interval === pollInterval) {
            return;
        }
        __DEV__ ? invariant$2(pollInterval, 'Attempted to start a polling query without a polling interval.') : invariant$2(pollInterval, 13);
        var info = pollingInfo || (this.pollingInfo = {});
        info.interval = pollInterval;
        var maybeFetch = function () {
            if (_this.pollingInfo) {
                if (!isNetworkRequestInFlight(_this.queryInfo.networkStatus)) {
                    _this.reobserve({
                        fetchPolicy: _this.options.initialFetchPolicy === 'no-cache' ? 'no-cache' : 'network-only',
                    }, NetworkStatus.poll).then(poll, poll);
                }
                else {
                    poll();
                }
            }
        };
        var poll = function () {
            var info = _this.pollingInfo;
            if (info) {
                clearTimeout(info.timeout);
                info.timeout = setTimeout(maybeFetch, info.interval);
            }
        };
        poll();
    };
    ObservableQuery.prototype.updateLastResult = function (newResult, variables) {
        if (variables === void 0) { variables = this.variables; }
        this.last = __assign$2(__assign$2({}, this.last), { result: this.queryManager.assumeImmutableResults
                ? newResult
                : cloneDeep(newResult), variables: variables });
        if (!isNonEmptyArray(newResult.errors)) {
            delete this.last.error;
        }
        return this.last;
    };
    ObservableQuery.prototype.reobserve = function (newOptions, newNetworkStatus) {
        var _this = this;
        this.isTornDown = false;
        var useDisposableConcast = newNetworkStatus === NetworkStatus.refetch ||
            newNetworkStatus === NetworkStatus.fetchMore ||
            newNetworkStatus === NetworkStatus.poll;
        var oldVariables = this.options.variables;
        var oldFetchPolicy = this.options.fetchPolicy;
        var mergedOptions = compact(this.options, newOptions || {});
        var options = useDisposableConcast
            ? mergedOptions
            : assign(this.options, mergedOptions);
        if (!useDisposableConcast) {
            this.updatePolling();
            if (newOptions &&
                newOptions.variables &&
                !equal(newOptions.variables, oldVariables) &&
                options.fetchPolicy !== "standby" &&
                options.fetchPolicy === oldFetchPolicy) {
                this.applyNextFetchPolicy("variables-changed", options);
                if (newNetworkStatus === void 0) {
                    newNetworkStatus = NetworkStatus.setVariables;
                }
            }
        }
        var variables = options.variables && __assign$2({}, options.variables);
        var concast = this.fetch(options, newNetworkStatus);
        var observer = {
            next: function (result) {
                _this.reportResult(result, variables);
            },
            error: function (error) {
                _this.reportError(error, variables);
            },
        };
        if (!useDisposableConcast) {
            if (this.concast && this.observer) {
                this.concast.removeObserver(this.observer);
            }
            this.concast = concast;
            this.observer = observer;
        }
        concast.addObserver(observer);
        return concast.promise;
    };
    ObservableQuery.prototype.observe = function () {
        this.reportResult(this.getCurrentResult(false), this.variables);
    };
    ObservableQuery.prototype.reportResult = function (result, variables) {
        var lastError = this.getLastError();
        if (lastError || this.isDifferentFromLastResult(result, variables)) {
            if (lastError || !result.partial || this.options.returnPartialData) {
                this.updateLastResult(result, variables);
            }
            iterateObserversSafely(this.observers, 'next', result);
        }
    };
    ObservableQuery.prototype.reportError = function (error, variables) {
        var errorResult = __assign$2(__assign$2({}, this.getLastResult()), { error: error, errors: error.graphQLErrors, networkStatus: NetworkStatus.error, loading: false });
        this.updateLastResult(errorResult, variables);
        iterateObserversSafely(this.observers, 'error', this.last.error = error);
    };
    ObservableQuery.prototype.hasObservers = function () {
        return this.observers.size > 0;
    };
    ObservableQuery.prototype.tearDownQuery = function () {
        if (this.isTornDown)
            return;
        if (this.concast && this.observer) {
            this.concast.removeObserver(this.observer);
            delete this.concast;
            delete this.observer;
        }
        this.stopPolling();
        this.subscriptions.forEach(function (sub) { return sub.unsubscribe(); });
        this.subscriptions.clear();
        this.queryManager.stopQuery(this.queryId);
        this.observers.clear();
        this.isTornDown = true;
    };
    return ObservableQuery;
}(Observable));
fixObservableSubclass(ObservableQuery);
function reobserveCacheFirst(obsQuery) {
    var _a = obsQuery.options, fetchPolicy = _a.fetchPolicy, nextFetchPolicy = _a.nextFetchPolicy;
    if (fetchPolicy === "cache-and-network" ||
        fetchPolicy === "network-only") {
        return obsQuery.reobserve({
            fetchPolicy: "cache-first",
            nextFetchPolicy: function () {
                this.nextFetchPolicy = nextFetchPolicy;
                if (typeof nextFetchPolicy === "function") {
                    return nextFetchPolicy.apply(this, arguments);
                }
                return fetchPolicy;
            },
        });
    }
    return obsQuery.reobserve();
}
function defaultSubscriptionObserverErrorCallback(error) {
    __DEV__ && invariant$2.error('Unhandled error', error.message, error.stack);
}
function logMissingFieldErrors(missing) {
    if (__DEV__ && missing) {
        __DEV__ && invariant$2.debug("Missing cache result fields: ".concat(JSON.stringify(missing)), missing);
    }
}

var LocalState = (function () {
    function LocalState(_a) {
        var cache = _a.cache, client = _a.client, resolvers = _a.resolvers, fragmentMatcher = _a.fragmentMatcher;
        this.selectionsToResolveCache = new WeakMap();
        this.cache = cache;
        if (client) {
            this.client = client;
        }
        if (resolvers) {
            this.addResolvers(resolvers);
        }
        if (fragmentMatcher) {
            this.setFragmentMatcher(fragmentMatcher);
        }
    }
    LocalState.prototype.addResolvers = function (resolvers) {
        var _this = this;
        this.resolvers = this.resolvers || {};
        if (Array.isArray(resolvers)) {
            resolvers.forEach(function (resolverGroup) {
                _this.resolvers = mergeDeep(_this.resolvers, resolverGroup);
            });
        }
        else {
            this.resolvers = mergeDeep(this.resolvers, resolvers);
        }
    };
    LocalState.prototype.setResolvers = function (resolvers) {
        this.resolvers = {};
        this.addResolvers(resolvers);
    };
    LocalState.prototype.getResolvers = function () {
        return this.resolvers || {};
    };
    LocalState.prototype.runResolvers = function (_a) {
        var document = _a.document, remoteResult = _a.remoteResult, context = _a.context, variables = _a.variables, _b = _a.onlyRunForcedResolvers, onlyRunForcedResolvers = _b === void 0 ? false : _b;
        return __awaiter$1(this, void 0, void 0, function () {
            return __generator$1(this, function (_c) {
                if (document) {
                    return [2, this.resolveDocument(document, remoteResult.data, context, variables, this.fragmentMatcher, onlyRunForcedResolvers).then(function (localResult) { return (__assign$2(__assign$2({}, remoteResult), { data: localResult.result })); })];
                }
                return [2, remoteResult];
            });
        });
    };
    LocalState.prototype.setFragmentMatcher = function (fragmentMatcher) {
        this.fragmentMatcher = fragmentMatcher;
    };
    LocalState.prototype.getFragmentMatcher = function () {
        return this.fragmentMatcher;
    };
    LocalState.prototype.clientQuery = function (document) {
        if (hasDirectives(['client'], document)) {
            if (this.resolvers) {
                return document;
            }
        }
        return null;
    };
    LocalState.prototype.serverQuery = function (document) {
        return removeClientSetsFromDocument(document);
    };
    LocalState.prototype.prepareContext = function (context) {
        var cache = this.cache;
        return __assign$2(__assign$2({}, context), { cache: cache, getCacheKey: function (obj) {
                return cache.identify(obj);
            } });
    };
    LocalState.prototype.addExportedVariables = function (document, variables, context) {
        if (variables === void 0) { variables = {}; }
        if (context === void 0) { context = {}; }
        return __awaiter$1(this, void 0, void 0, function () {
            return __generator$1(this, function (_a) {
                if (document) {
                    return [2, this.resolveDocument(document, this.buildRootValueFromCache(document, variables) || {}, this.prepareContext(context), variables).then(function (data) { return (__assign$2(__assign$2({}, variables), data.exportedVariables)); })];
                }
                return [2, __assign$2({}, variables)];
            });
        });
    };
    LocalState.prototype.shouldForceResolvers = function (document) {
        var forceResolvers = false;
        visit(document, {
            Directive: {
                enter: function (node) {
                    if (node.name.value === 'client' && node.arguments) {
                        forceResolvers = node.arguments.some(function (arg) {
                            return arg.name.value === 'always' &&
                                arg.value.kind === 'BooleanValue' &&
                                arg.value.value === true;
                        });
                        if (forceResolvers) {
                            return BREAK;
                        }
                    }
                },
            },
        });
        return forceResolvers;
    };
    LocalState.prototype.buildRootValueFromCache = function (document, variables) {
        return this.cache.diff({
            query: buildQueryFromSelectionSet(document),
            variables: variables,
            returnPartialData: true,
            optimistic: false,
        }).result;
    };
    LocalState.prototype.resolveDocument = function (document, rootValue, context, variables, fragmentMatcher, onlyRunForcedResolvers) {
        if (context === void 0) { context = {}; }
        if (variables === void 0) { variables = {}; }
        if (fragmentMatcher === void 0) { fragmentMatcher = function () { return true; }; }
        if (onlyRunForcedResolvers === void 0) { onlyRunForcedResolvers = false; }
        return __awaiter$1(this, void 0, void 0, function () {
            var mainDefinition, fragments, fragmentMap, selectionsToResolve, definitionOperation, defaultOperationType, _a, cache, client, execContext, isClientFieldDescendant;
            return __generator$1(this, function (_b) {
                mainDefinition = getMainDefinition(document);
                fragments = getFragmentDefinitions(document);
                fragmentMap = createFragmentMap(fragments);
                selectionsToResolve = this.collectSelectionsToResolve(mainDefinition, fragmentMap);
                definitionOperation = mainDefinition.operation;
                defaultOperationType = definitionOperation
                    ? definitionOperation.charAt(0).toUpperCase() +
                        definitionOperation.slice(1)
                    : 'Query';
                _a = this, cache = _a.cache, client = _a.client;
                execContext = {
                    fragmentMap: fragmentMap,
                    context: __assign$2(__assign$2({}, context), { cache: cache, client: client }),
                    variables: variables,
                    fragmentMatcher: fragmentMatcher,
                    defaultOperationType: defaultOperationType,
                    exportedVariables: {},
                    selectionsToResolve: selectionsToResolve,
                    onlyRunForcedResolvers: onlyRunForcedResolvers,
                };
                isClientFieldDescendant = false;
                return [2, this.resolveSelectionSet(mainDefinition.selectionSet, isClientFieldDescendant, rootValue, execContext).then(function (result) { return ({
                        result: result,
                        exportedVariables: execContext.exportedVariables,
                    }); })];
            });
        });
    };
    LocalState.prototype.resolveSelectionSet = function (selectionSet, isClientFieldDescendant, rootValue, execContext) {
        return __awaiter$1(this, void 0, void 0, function () {
            var fragmentMap, context, variables, resultsToMerge, execute;
            var _this = this;
            return __generator$1(this, function (_a) {
                fragmentMap = execContext.fragmentMap, context = execContext.context, variables = execContext.variables;
                resultsToMerge = [rootValue];
                execute = function (selection) { return __awaiter$1(_this, void 0, void 0, function () {
                    var fragment, typeCondition;
                    return __generator$1(this, function (_a) {
                        if (!isClientFieldDescendant && !execContext.selectionsToResolve.has(selection)) {
                            return [2];
                        }
                        if (!shouldInclude(selection, variables)) {
                            return [2];
                        }
                        if (isField(selection)) {
                            return [2, this.resolveField(selection, isClientFieldDescendant, rootValue, execContext).then(function (fieldResult) {
                                    var _a;
                                    if (typeof fieldResult !== 'undefined') {
                                        resultsToMerge.push((_a = {},
                                            _a[resultKeyNameFromField(selection)] = fieldResult,
                                            _a));
                                    }
                                })];
                        }
                        if (isInlineFragment(selection)) {
                            fragment = selection;
                        }
                        else {
                            fragment = fragmentMap[selection.name.value];
                            __DEV__ ? invariant$2(fragment, "No fragment named ".concat(selection.name.value)) : invariant$2(fragment, 11);
                        }
                        if (fragment && fragment.typeCondition) {
                            typeCondition = fragment.typeCondition.name.value;
                            if (execContext.fragmentMatcher(rootValue, typeCondition, context)) {
                                return [2, this.resolveSelectionSet(fragment.selectionSet, isClientFieldDescendant, rootValue, execContext).then(function (fragmentResult) {
                                        resultsToMerge.push(fragmentResult);
                                    })];
                            }
                        }
                        return [2];
                    });
                }); };
                return [2, Promise.all(selectionSet.selections.map(execute)).then(function () {
                        return mergeDeepArray(resultsToMerge);
                    })];
            });
        });
    };
    LocalState.prototype.resolveField = function (field, isClientFieldDescendant, rootValue, execContext) {
        return __awaiter$1(this, void 0, void 0, function () {
            var variables, fieldName, aliasedFieldName, aliasUsed, defaultResult, resultPromise, resolverType, resolverMap, resolve;
            var _this = this;
            return __generator$1(this, function (_a) {
                if (!rootValue) {
                    return [2, null];
                }
                variables = execContext.variables;
                fieldName = field.name.value;
                aliasedFieldName = resultKeyNameFromField(field);
                aliasUsed = fieldName !== aliasedFieldName;
                defaultResult = rootValue[aliasedFieldName] || rootValue[fieldName];
                resultPromise = Promise.resolve(defaultResult);
                if (!execContext.onlyRunForcedResolvers ||
                    this.shouldForceResolvers(field)) {
                    resolverType = rootValue.__typename || execContext.defaultOperationType;
                    resolverMap = this.resolvers && this.resolvers[resolverType];
                    if (resolverMap) {
                        resolve = resolverMap[aliasUsed ? fieldName : aliasedFieldName];
                        if (resolve) {
                            resultPromise = Promise.resolve(cacheSlot.withValue(this.cache, resolve, [
                                rootValue,
                                argumentsObjectFromField(field, variables),
                                execContext.context,
                                { field: field, fragmentMap: execContext.fragmentMap },
                            ]));
                        }
                    }
                }
                return [2, resultPromise.then(function (result) {
                        var _a, _b;
                        if (result === void 0) { result = defaultResult; }
                        if (field.directives) {
                            field.directives.forEach(function (directive) {
                                if (directive.name.value === 'export' && directive.arguments) {
                                    directive.arguments.forEach(function (arg) {
                                        if (arg.name.value === 'as' && arg.value.kind === 'StringValue') {
                                            execContext.exportedVariables[arg.value.value] = result;
                                        }
                                    });
                                }
                            });
                        }
                        if (!field.selectionSet) {
                            return result;
                        }
                        if (result == null) {
                            return result;
                        }
                        var isClientField = (_b = (_a = field.directives) === null || _a === void 0 ? void 0 : _a.some(function (d) { return d.name.value === 'client'; })) !== null && _b !== void 0 ? _b : false;
                        if (Array.isArray(result)) {
                            return _this.resolveSubSelectedArray(field, isClientFieldDescendant || isClientField, result, execContext);
                        }
                        if (field.selectionSet) {
                            return _this.resolveSelectionSet(field.selectionSet, isClientFieldDescendant || isClientField, result, execContext);
                        }
                    })];
            });
        });
    };
    LocalState.prototype.resolveSubSelectedArray = function (field, isClientFieldDescendant, result, execContext) {
        var _this = this;
        return Promise.all(result.map(function (item) {
            if (item === null) {
                return null;
            }
            if (Array.isArray(item)) {
                return _this.resolveSubSelectedArray(field, isClientFieldDescendant, item, execContext);
            }
            if (field.selectionSet) {
                return _this.resolveSelectionSet(field.selectionSet, isClientFieldDescendant, item, execContext);
            }
        }));
    };
    LocalState.prototype.collectSelectionsToResolve = function (mainDefinition, fragmentMap) {
        var isSingleASTNode = function (node) { return !Array.isArray(node); };
        var selectionsToResolveCache = this.selectionsToResolveCache;
        function collectByDefinition(definitionNode) {
            if (!selectionsToResolveCache.has(definitionNode)) {
                var matches_1 = new Set();
                selectionsToResolveCache.set(definitionNode, matches_1);
                visit(definitionNode, {
                    Directive: function (node, _, __, ___, ancestors) {
                        if (node.name.value === 'client') {
                            ancestors.forEach(function (node) {
                                if (isSingleASTNode(node) && isSelectionNode(node)) {
                                    matches_1.add(node);
                                }
                            });
                        }
                    },
                    FragmentSpread: function (spread, _, __, ___, ancestors) {
                        var fragment = fragmentMap[spread.name.value];
                        __DEV__ ? invariant$2(fragment, "No fragment named ".concat(spread.name.value)) : invariant$2(fragment, 12);
                        var fragmentSelections = collectByDefinition(fragment);
                        if (fragmentSelections.size > 0) {
                            ancestors.forEach(function (node) {
                                if (isSingleASTNode(node) && isSelectionNode(node)) {
                                    matches_1.add(node);
                                }
                            });
                            matches_1.add(spread);
                            fragmentSelections.forEach(function (selection) {
                                matches_1.add(selection);
                            });
                        }
                    }
                });
            }
            return selectionsToResolveCache.get(definitionNode);
        }
        return collectByDefinition(mainDefinition);
    };
    return LocalState;
}());

var destructiveMethodCounts = new (canUseWeakMap ? WeakMap : Map)();
function wrapDestructiveCacheMethod(cache, methodName) {
    var original = cache[methodName];
    if (typeof original === "function") {
        cache[methodName] = function () {
            destructiveMethodCounts.set(cache, (destructiveMethodCounts.get(cache) + 1) % 1e15);
            return original.apply(this, arguments);
        };
    }
}
function cancelNotifyTimeout(info) {
    if (info["notifyTimeout"]) {
        clearTimeout(info["notifyTimeout"]);
        info["notifyTimeout"] = void 0;
    }
}
var QueryInfo = (function () {
    function QueryInfo(queryManager, queryId) {
        if (queryId === void 0) { queryId = queryManager.generateQueryId(); }
        this.queryId = queryId;
        this.listeners = new Set();
        this.document = null;
        this.lastRequestId = 1;
        this.subscriptions = new Set();
        this.stopped = false;
        this.dirty = false;
        this.observableQuery = null;
        var cache = this.cache = queryManager.cache;
        if (!destructiveMethodCounts.has(cache)) {
            destructiveMethodCounts.set(cache, 0);
            wrapDestructiveCacheMethod(cache, "evict");
            wrapDestructiveCacheMethod(cache, "modify");
            wrapDestructiveCacheMethod(cache, "reset");
        }
    }
    QueryInfo.prototype.init = function (query) {
        var networkStatus = query.networkStatus || NetworkStatus.loading;
        if (this.variables &&
            this.networkStatus !== NetworkStatus.loading &&
            !equal(this.variables, query.variables)) {
            networkStatus = NetworkStatus.setVariables;
        }
        if (!equal(query.variables, this.variables)) {
            this.lastDiff = void 0;
        }
        Object.assign(this, {
            document: query.document,
            variables: query.variables,
            networkError: null,
            graphQLErrors: this.graphQLErrors || [],
            networkStatus: networkStatus,
        });
        if (query.observableQuery) {
            this.setObservableQuery(query.observableQuery);
        }
        if (query.lastRequestId) {
            this.lastRequestId = query.lastRequestId;
        }
        return this;
    };
    QueryInfo.prototype.reset = function () {
        cancelNotifyTimeout(this);
        this.dirty = false;
    };
    QueryInfo.prototype.getDiff = function (variables) {
        if (variables === void 0) { variables = this.variables; }
        var options = this.getDiffOptions(variables);
        if (this.lastDiff && equal(options, this.lastDiff.options)) {
            return this.lastDiff.diff;
        }
        this.updateWatch(this.variables = variables);
        var oq = this.observableQuery;
        if (oq && oq.options.fetchPolicy === "no-cache") {
            return { complete: false };
        }
        var diff = this.cache.diff(options);
        this.updateLastDiff(diff, options);
        return diff;
    };
    QueryInfo.prototype.updateLastDiff = function (diff, options) {
        this.lastDiff = diff ? {
            diff: diff,
            options: options || this.getDiffOptions(),
        } : void 0;
    };
    QueryInfo.prototype.getDiffOptions = function (variables) {
        var _a;
        if (variables === void 0) { variables = this.variables; }
        return {
            query: this.document,
            variables: variables,
            returnPartialData: true,
            optimistic: true,
            canonizeResults: (_a = this.observableQuery) === null || _a === void 0 ? void 0 : _a.options.canonizeResults,
        };
    };
    QueryInfo.prototype.setDiff = function (diff) {
        var _this = this;
        var oldDiff = this.lastDiff && this.lastDiff.diff;
        this.updateLastDiff(diff);
        if (!this.dirty &&
            !equal(oldDiff && oldDiff.result, diff && diff.result)) {
            this.dirty = true;
            if (!this.notifyTimeout) {
                this.notifyTimeout = setTimeout(function () { return _this.notify(); }, 0);
            }
        }
    };
    QueryInfo.prototype.setObservableQuery = function (oq) {
        var _this = this;
        if (oq === this.observableQuery)
            return;
        if (this.oqListener) {
            this.listeners.delete(this.oqListener);
        }
        this.observableQuery = oq;
        if (oq) {
            oq["queryInfo"] = this;
            this.listeners.add(this.oqListener = function () {
                var diff = _this.getDiff();
                if (diff.fromOptimisticTransaction) {
                    oq["observe"]();
                }
                else {
                    reobserveCacheFirst(oq);
                }
            });
        }
        else {
            delete this.oqListener;
        }
    };
    QueryInfo.prototype.notify = function () {
        var _this = this;
        cancelNotifyTimeout(this);
        if (this.shouldNotify()) {
            this.listeners.forEach(function (listener) { return listener(_this); });
        }
        this.dirty = false;
    };
    QueryInfo.prototype.shouldNotify = function () {
        if (!this.dirty || !this.listeners.size) {
            return false;
        }
        if (isNetworkRequestInFlight(this.networkStatus) &&
            this.observableQuery) {
            var fetchPolicy = this.observableQuery.options.fetchPolicy;
            if (fetchPolicy !== "cache-only" &&
                fetchPolicy !== "cache-and-network") {
                return false;
            }
        }
        return true;
    };
    QueryInfo.prototype.stop = function () {
        if (!this.stopped) {
            this.stopped = true;
            this.reset();
            this.cancel();
            this.cancel = QueryInfo.prototype.cancel;
            this.subscriptions.forEach(function (sub) { return sub.unsubscribe(); });
            var oq = this.observableQuery;
            if (oq)
                oq.stopPolling();
        }
    };
    QueryInfo.prototype.cancel = function () { };
    QueryInfo.prototype.updateWatch = function (variables) {
        var _this = this;
        if (variables === void 0) { variables = this.variables; }
        var oq = this.observableQuery;
        if (oq && oq.options.fetchPolicy === "no-cache") {
            return;
        }
        var watchOptions = __assign$2(__assign$2({}, this.getDiffOptions(variables)), { watcher: this, callback: function (diff) { return _this.setDiff(diff); } });
        if (!this.lastWatch ||
            !equal(watchOptions, this.lastWatch)) {
            this.cancel();
            this.cancel = this.cache.watch(this.lastWatch = watchOptions);
        }
    };
    QueryInfo.prototype.resetLastWrite = function () {
        this.lastWrite = void 0;
    };
    QueryInfo.prototype.shouldWrite = function (result, variables) {
        var lastWrite = this.lastWrite;
        return !(lastWrite &&
            lastWrite.dmCount === destructiveMethodCounts.get(this.cache) &&
            equal(variables, lastWrite.variables) &&
            equal(result.data, lastWrite.result.data));
    };
    QueryInfo.prototype.markResult = function (result, document, options, cacheWriteBehavior) {
        var _this = this;
        var merger = new DeepMerger();
        var graphQLErrors = isNonEmptyArray(result.errors)
            ? result.errors.slice(0)
            : [];
        this.reset();
        if ('incremental' in result && isNonEmptyArray(result.incremental)) {
            var mergedData = mergeIncrementalData(this.getDiff().result, result);
            result.data = mergedData;
        }
        else if ('hasNext' in result && result.hasNext) {
            var diff = this.getDiff();
            result.data = merger.merge(diff.result, result.data);
        }
        this.graphQLErrors = graphQLErrors;
        if (options.fetchPolicy === 'no-cache') {
            this.updateLastDiff({ result: result.data, complete: true }, this.getDiffOptions(options.variables));
        }
        else if (cacheWriteBehavior !== 0) {
            if (shouldWriteResult(result, options.errorPolicy)) {
                this.cache.performTransaction(function (cache) {
                    if (_this.shouldWrite(result, options.variables)) {
                        cache.writeQuery({
                            query: document,
                            data: result.data,
                            variables: options.variables,
                            overwrite: cacheWriteBehavior === 1,
                        });
                        _this.lastWrite = {
                            result: result,
                            variables: options.variables,
                            dmCount: destructiveMethodCounts.get(_this.cache),
                        };
                    }
                    else {
                        if (_this.lastDiff &&
                            _this.lastDiff.diff.complete) {
                            result.data = _this.lastDiff.diff.result;
                            return;
                        }
                    }
                    var diffOptions = _this.getDiffOptions(options.variables);
                    var diff = cache.diff(diffOptions);
                    if (!_this.stopped) {
                        _this.updateWatch(options.variables);
                    }
                    _this.updateLastDiff(diff, diffOptions);
                    if (diff.complete) {
                        result.data = diff.result;
                    }
                });
            }
            else {
                this.lastWrite = void 0;
            }
        }
    };
    QueryInfo.prototype.markReady = function () {
        this.networkError = null;
        return this.networkStatus = NetworkStatus.ready;
    };
    QueryInfo.prototype.markError = function (error) {
        this.networkStatus = NetworkStatus.error;
        this.lastWrite = void 0;
        this.reset();
        if (error.graphQLErrors) {
            this.graphQLErrors = error.graphQLErrors;
        }
        if (error.networkError) {
            this.networkError = error.networkError;
        }
        return error;
    };
    return QueryInfo;
}());
function shouldWriteResult(result, errorPolicy) {
    if (errorPolicy === void 0) { errorPolicy = "none"; }
    var ignoreErrors = errorPolicy === "ignore" ||
        errorPolicy === "all";
    var writeWithErrors = !graphQLResultHasError(result);
    if (!writeWithErrors && ignoreErrors && result.data) {
        writeWithErrors = true;
    }
    return writeWithErrors;
}

var hasOwnProperty = Object.prototype.hasOwnProperty;
var QueryManager = (function () {
    function QueryManager(_a) {
        var cache = _a.cache, link = _a.link, defaultOptions = _a.defaultOptions, _b = _a.queryDeduplication, queryDeduplication = _b === void 0 ? false : _b, onBroadcast = _a.onBroadcast, _c = _a.ssrMode, ssrMode = _c === void 0 ? false : _c, _d = _a.clientAwareness, clientAwareness = _d === void 0 ? {} : _d, localState = _a.localState, assumeImmutableResults = _a.assumeImmutableResults;
        this.clientAwareness = {};
        this.queries = new Map();
        this.fetchCancelFns = new Map();
        this.transformCache = new (canUseWeakMap ? WeakMap : Map)();
        this.queryIdCounter = 1;
        this.requestIdCounter = 1;
        this.mutationIdCounter = 1;
        this.inFlightLinkObservables = new Map();
        this.cache = cache;
        this.link = link;
        this.defaultOptions = defaultOptions || Object.create(null);
        this.queryDeduplication = queryDeduplication;
        this.clientAwareness = clientAwareness;
        this.localState = localState || new LocalState({ cache: cache });
        this.ssrMode = ssrMode;
        this.assumeImmutableResults = !!assumeImmutableResults;
        if ((this.onBroadcast = onBroadcast)) {
            this.mutationStore = Object.create(null);
        }
    }
    QueryManager.prototype.stop = function () {
        var _this = this;
        this.queries.forEach(function (_info, queryId) {
            _this.stopQueryNoBroadcast(queryId);
        });
        this.cancelPendingFetches(__DEV__ ? new InvariantError('QueryManager stopped while query was in flight') : new InvariantError(14));
    };
    QueryManager.prototype.cancelPendingFetches = function (error) {
        this.fetchCancelFns.forEach(function (cancel) { return cancel(error); });
        this.fetchCancelFns.clear();
    };
    QueryManager.prototype.mutate = function (_a) {
        var _b, _c;
        var mutation = _a.mutation, variables = _a.variables, optimisticResponse = _a.optimisticResponse, updateQueries = _a.updateQueries, _d = _a.refetchQueries, refetchQueries = _d === void 0 ? [] : _d, _e = _a.awaitRefetchQueries, awaitRefetchQueries = _e === void 0 ? false : _e, updateWithProxyFn = _a.update, onQueryUpdated = _a.onQueryUpdated, _f = _a.fetchPolicy, fetchPolicy = _f === void 0 ? ((_b = this.defaultOptions.mutate) === null || _b === void 0 ? void 0 : _b.fetchPolicy) || "network-only" : _f, _g = _a.errorPolicy, errorPolicy = _g === void 0 ? ((_c = this.defaultOptions.mutate) === null || _c === void 0 ? void 0 : _c.errorPolicy) || "none" : _g, keepRootFields = _a.keepRootFields, context = _a.context;
        return __awaiter$1(this, void 0, void 0, function () {
            var mutationId, _h, document, hasClientExports, mutationStoreValue, self;
            return __generator$1(this, function (_j) {
                switch (_j.label) {
                    case 0:
                        __DEV__ ? invariant$2(mutation, 'mutation option is required. You must specify your GraphQL document in the mutation option.') : invariant$2(mutation, 15);
                        __DEV__ ? invariant$2(fetchPolicy === 'network-only' ||
                            fetchPolicy === 'no-cache', "Mutations support only 'network-only' or 'no-cache' fetchPolicy strings. The default `network-only` behavior automatically writes mutation results to the cache. Passing `no-cache` skips the cache write.") : invariant$2(fetchPolicy === 'network-only' ||
                            fetchPolicy === 'no-cache', 16);
                        mutationId = this.generateMutationId();
                        _h = this.transform(mutation), document = _h.document, hasClientExports = _h.hasClientExports;
                        mutation = this.cache.transformForLink(document);
                        variables = this.getVariables(mutation, variables);
                        if (!hasClientExports) return [3, 2];
                        return [4, this.localState.addExportedVariables(mutation, variables, context)];
                    case 1:
                        variables = (_j.sent());
                        _j.label = 2;
                    case 2:
                        mutationStoreValue = this.mutationStore &&
                            (this.mutationStore[mutationId] = {
                                mutation: mutation,
                                variables: variables,
                                loading: true,
                                error: null,
                            });
                        if (optimisticResponse) {
                            this.markMutationOptimistic(optimisticResponse, {
                                mutationId: mutationId,
                                document: mutation,
                                variables: variables,
                                fetchPolicy: fetchPolicy,
                                errorPolicy: errorPolicy,
                                context: context,
                                updateQueries: updateQueries,
                                update: updateWithProxyFn,
                                keepRootFields: keepRootFields,
                            });
                        }
                        this.broadcastQueries();
                        self = this;
                        return [2, new Promise(function (resolve, reject) {
                                return asyncMap(self.getObservableFromLink(mutation, __assign$2(__assign$2({}, context), { optimisticResponse: optimisticResponse }), variables, false), function (result) {
                                    if (graphQLResultHasError(result) && errorPolicy === 'none') {
                                        throw new ApolloError({
                                            graphQLErrors: getGraphQLErrorsFromResult(result),
                                        });
                                    }
                                    if (mutationStoreValue) {
                                        mutationStoreValue.loading = false;
                                        mutationStoreValue.error = null;
                                    }
                                    var storeResult = __assign$2({}, result);
                                    if (typeof refetchQueries === "function") {
                                        refetchQueries = refetchQueries(storeResult);
                                    }
                                    if (errorPolicy === 'ignore' &&
                                        graphQLResultHasError(storeResult)) {
                                        delete storeResult.errors;
                                    }
                                    return self.markMutationResult({
                                        mutationId: mutationId,
                                        result: storeResult,
                                        document: mutation,
                                        variables: variables,
                                        fetchPolicy: fetchPolicy,
                                        errorPolicy: errorPolicy,
                                        context: context,
                                        update: updateWithProxyFn,
                                        updateQueries: updateQueries,
                                        awaitRefetchQueries: awaitRefetchQueries,
                                        refetchQueries: refetchQueries,
                                        removeOptimistic: optimisticResponse ? mutationId : void 0,
                                        onQueryUpdated: onQueryUpdated,
                                        keepRootFields: keepRootFields,
                                    });
                                }).subscribe({
                                    next: function (storeResult) {
                                        self.broadcastQueries();
                                        if (!('hasNext' in storeResult) || storeResult.hasNext === false) {
                                            resolve(storeResult);
                                        }
                                    },
                                    error: function (err) {
                                        if (mutationStoreValue) {
                                            mutationStoreValue.loading = false;
                                            mutationStoreValue.error = err;
                                        }
                                        if (optimisticResponse) {
                                            self.cache.removeOptimistic(mutationId);
                                        }
                                        self.broadcastQueries();
                                        reject(err instanceof ApolloError ? err : new ApolloError({
                                            networkError: err,
                                        }));
                                    },
                                });
                            })];
                }
            });
        });
    };
    QueryManager.prototype.markMutationResult = function (mutation, cache) {
        var _this = this;
        if (cache === void 0) { cache = this.cache; }
        var result = mutation.result;
        var cacheWrites = [];
        var skipCache = mutation.fetchPolicy === "no-cache";
        if (!skipCache && shouldWriteResult(result, mutation.errorPolicy)) {
            if (!isExecutionPatchIncrementalResult(result)) {
                cacheWrites.push({
                    result: result.data,
                    dataId: 'ROOT_MUTATION',
                    query: mutation.document,
                    variables: mutation.variables,
                });
            }
            if (isExecutionPatchIncrementalResult(result) && isNonEmptyArray(result.incremental)) {
                var diff = cache.diff({
                    id: "ROOT_MUTATION",
                    query: this.transform(mutation.document).asQuery,
                    variables: mutation.variables,
                    optimistic: false,
                    returnPartialData: true,
                });
                var mergedData = void 0;
                if (diff.result) {
                    mergedData = mergeIncrementalData(diff.result, result);
                }
                if (typeof mergedData !== 'undefined') {
                    result.data = mergedData;
                    cacheWrites.push({
                        result: mergedData,
                        dataId: 'ROOT_MUTATION',
                        query: mutation.document,
                        variables: mutation.variables,
                    });
                }
            }
            var updateQueries_1 = mutation.updateQueries;
            if (updateQueries_1) {
                this.queries.forEach(function (_a, queryId) {
                    var observableQuery = _a.observableQuery;
                    var queryName = observableQuery && observableQuery.queryName;
                    if (!queryName || !hasOwnProperty.call(updateQueries_1, queryName)) {
                        return;
                    }
                    var updater = updateQueries_1[queryName];
                    var _b = _this.queries.get(queryId), document = _b.document, variables = _b.variables;
                    var _c = cache.diff({
                        query: document,
                        variables: variables,
                        returnPartialData: true,
                        optimistic: false,
                    }), currentQueryResult = _c.result, complete = _c.complete;
                    if (complete && currentQueryResult) {
                        var nextQueryResult = updater(currentQueryResult, {
                            mutationResult: result,
                            queryName: document && getOperationName(document) || void 0,
                            queryVariables: variables,
                        });
                        if (nextQueryResult) {
                            cacheWrites.push({
                                result: nextQueryResult,
                                dataId: 'ROOT_QUERY',
                                query: document,
                                variables: variables,
                            });
                        }
                    }
                });
            }
        }
        if (cacheWrites.length > 0 ||
            mutation.refetchQueries ||
            mutation.update ||
            mutation.onQueryUpdated ||
            mutation.removeOptimistic) {
            var results_1 = [];
            this.refetchQueries({
                updateCache: function (cache) {
                    if (!skipCache) {
                        cacheWrites.forEach(function (write) { return cache.write(write); });
                    }
                    var update = mutation.update;
                    var isFinalResult = !isExecutionPatchResult(result) ||
                        (isExecutionPatchIncrementalResult(result) && !result.hasNext);
                    if (update) {
                        if (!skipCache) {
                            var diff = cache.diff({
                                id: "ROOT_MUTATION",
                                query: _this.transform(mutation.document).asQuery,
                                variables: mutation.variables,
                                optimistic: false,
                                returnPartialData: true,
                            });
                            if (diff.complete) {
                                result = __assign$2(__assign$2({}, result), { data: diff.result });
                                if ('incremental' in result) {
                                    delete result.incremental;
                                }
                                if ('hasNext' in result) {
                                    delete result.hasNext;
                                }
                            }
                        }
                        if (isFinalResult) {
                            update(cache, result, {
                                context: mutation.context,
                                variables: mutation.variables,
                            });
                        }
                    }
                    if (!skipCache && !mutation.keepRootFields && isFinalResult) {
                        cache.modify({
                            id: 'ROOT_MUTATION',
                            fields: function (value, _a) {
                                var fieldName = _a.fieldName, DELETE = _a.DELETE;
                                return fieldName === "__typename" ? value : DELETE;
                            },
                        });
                    }
                },
                include: mutation.refetchQueries,
                optimistic: false,
                removeOptimistic: mutation.removeOptimistic,
                onQueryUpdated: mutation.onQueryUpdated || null,
            }).forEach(function (result) { return results_1.push(result); });
            if (mutation.awaitRefetchQueries || mutation.onQueryUpdated) {
                return Promise.all(results_1).then(function () { return result; });
            }
        }
        return Promise.resolve(result);
    };
    QueryManager.prototype.markMutationOptimistic = function (optimisticResponse, mutation) {
        var _this = this;
        var data = typeof optimisticResponse === "function"
            ? optimisticResponse(mutation.variables)
            : optimisticResponse;
        return this.cache.recordOptimisticTransaction(function (cache) {
            try {
                _this.markMutationResult(__assign$2(__assign$2({}, mutation), { result: { data: data } }), cache);
            }
            catch (error) {
                __DEV__ && invariant$2.error(error);
            }
        }, mutation.mutationId);
    };
    QueryManager.prototype.fetchQuery = function (queryId, options, networkStatus) {
        return this.fetchQueryObservable(queryId, options, networkStatus).promise;
    };
    QueryManager.prototype.getQueryStore = function () {
        var store = Object.create(null);
        this.queries.forEach(function (info, queryId) {
            store[queryId] = {
                variables: info.variables,
                networkStatus: info.networkStatus,
                networkError: info.networkError,
                graphQLErrors: info.graphQLErrors,
            };
        });
        return store;
    };
    QueryManager.prototype.resetErrors = function (queryId) {
        var queryInfo = this.queries.get(queryId);
        if (queryInfo) {
            queryInfo.networkError = undefined;
            queryInfo.graphQLErrors = [];
        }
    };
    QueryManager.prototype.transform = function (document) {
        var transformCache = this.transformCache;
        if (!transformCache.has(document)) {
            var transformed = this.cache.transformDocument(document);
            var noConnection = removeConnectionDirectiveFromDocument(transformed);
            var clientQuery = this.localState.clientQuery(transformed);
            var serverQuery = noConnection && this.localState.serverQuery(noConnection);
            var cacheEntry_1 = {
                document: transformed,
                hasClientExports: hasClientExports(transformed),
                hasForcedResolvers: this.localState.shouldForceResolvers(transformed),
                clientQuery: clientQuery,
                serverQuery: serverQuery,
                defaultVars: getDefaultValues(getOperationDefinition(transformed)),
                asQuery: __assign$2(__assign$2({}, transformed), { definitions: transformed.definitions.map(function (def) {
                        if (def.kind === "OperationDefinition" &&
                            def.operation !== "query") {
                            return __assign$2(__assign$2({}, def), { operation: "query" });
                        }
                        return def;
                    }) })
            };
            var add = function (doc) {
                if (doc && !transformCache.has(doc)) {
                    transformCache.set(doc, cacheEntry_1);
                }
            };
            add(document);
            add(transformed);
            add(clientQuery);
            add(serverQuery);
        }
        return transformCache.get(document);
    };
    QueryManager.prototype.getVariables = function (document, variables) {
        return __assign$2(__assign$2({}, this.transform(document).defaultVars), variables);
    };
    QueryManager.prototype.watchQuery = function (options) {
        options = __assign$2(__assign$2({}, options), { variables: this.getVariables(options.query, options.variables) });
        if (typeof options.notifyOnNetworkStatusChange === 'undefined') {
            options.notifyOnNetworkStatusChange = false;
        }
        var queryInfo = new QueryInfo(this);
        var observable = new ObservableQuery({
            queryManager: this,
            queryInfo: queryInfo,
            options: options,
        });
        this.queries.set(observable.queryId, queryInfo);
        queryInfo.init({
            document: observable.query,
            observableQuery: observable,
            variables: observable.variables,
        });
        return observable;
    };
    QueryManager.prototype.query = function (options, queryId) {
        var _this = this;
        if (queryId === void 0) { queryId = this.generateQueryId(); }
        __DEV__ ? invariant$2(options.query, 'query option is required. You must specify your GraphQL document ' +
            'in the query option.') : invariant$2(options.query, 17);
        __DEV__ ? invariant$2(options.query.kind === 'Document', 'You must wrap the query string in a "gql" tag.') : invariant$2(options.query.kind === 'Document', 18);
        __DEV__ ? invariant$2(!options.returnPartialData, 'returnPartialData option only supported on watchQuery.') : invariant$2(!options.returnPartialData, 19);
        __DEV__ ? invariant$2(!options.pollInterval, 'pollInterval option only supported on watchQuery.') : invariant$2(!options.pollInterval, 20);
        return this.fetchQuery(queryId, options).finally(function () { return _this.stopQuery(queryId); });
    };
    QueryManager.prototype.generateQueryId = function () {
        return String(this.queryIdCounter++);
    };
    QueryManager.prototype.generateRequestId = function () {
        return this.requestIdCounter++;
    };
    QueryManager.prototype.generateMutationId = function () {
        return String(this.mutationIdCounter++);
    };
    QueryManager.prototype.stopQueryInStore = function (queryId) {
        this.stopQueryInStoreNoBroadcast(queryId);
        this.broadcastQueries();
    };
    QueryManager.prototype.stopQueryInStoreNoBroadcast = function (queryId) {
        var queryInfo = this.queries.get(queryId);
        if (queryInfo)
            queryInfo.stop();
    };
    QueryManager.prototype.clearStore = function (options) {
        if (options === void 0) { options = {
            discardWatches: true,
        }; }
        this.cancelPendingFetches(__DEV__ ? new InvariantError('Store reset while query was in flight (not completed in link chain)') : new InvariantError(21));
        this.queries.forEach(function (queryInfo) {
            if (queryInfo.observableQuery) {
                queryInfo.networkStatus = NetworkStatus.loading;
            }
            else {
                queryInfo.stop();
            }
        });
        if (this.mutationStore) {
            this.mutationStore = Object.create(null);
        }
        return this.cache.reset(options);
    };
    QueryManager.prototype.getObservableQueries = function (include) {
        var _this = this;
        if (include === void 0) { include = "active"; }
        var queries = new Map();
        var queryNamesAndDocs = new Map();
        var legacyQueryOptions = new Set();
        if (Array.isArray(include)) {
            include.forEach(function (desc) {
                if (typeof desc === "string") {
                    queryNamesAndDocs.set(desc, false);
                }
                else if (isDocumentNode(desc)) {
                    queryNamesAndDocs.set(_this.transform(desc).document, false);
                }
                else if (isNonNullObject(desc) && desc.query) {
                    legacyQueryOptions.add(desc);
                }
            });
        }
        this.queries.forEach(function (_a, queryId) {
            var oq = _a.observableQuery, document = _a.document;
            if (oq) {
                if (include === "all") {
                    queries.set(queryId, oq);
                    return;
                }
                var queryName = oq.queryName, fetchPolicy = oq.options.fetchPolicy;
                if (fetchPolicy === "standby" ||
                    (include === "active" && !oq.hasObservers())) {
                    return;
                }
                if (include === "active" ||
                    (queryName && queryNamesAndDocs.has(queryName)) ||
                    (document && queryNamesAndDocs.has(document))) {
                    queries.set(queryId, oq);
                    if (queryName)
                        queryNamesAndDocs.set(queryName, true);
                    if (document)
                        queryNamesAndDocs.set(document, true);
                }
            }
        });
        if (legacyQueryOptions.size) {
            legacyQueryOptions.forEach(function (options) {
                var queryId = makeUniqueId("legacyOneTimeQuery");
                var queryInfo = _this.getQuery(queryId).init({
                    document: options.query,
                    variables: options.variables,
                });
                var oq = new ObservableQuery({
                    queryManager: _this,
                    queryInfo: queryInfo,
                    options: __assign$2(__assign$2({}, options), { fetchPolicy: "network-only" }),
                });
                invariant$2(oq.queryId === queryId);
                queryInfo.setObservableQuery(oq);
                queries.set(queryId, oq);
            });
        }
        if (__DEV__ && queryNamesAndDocs.size) {
            queryNamesAndDocs.forEach(function (included, nameOrDoc) {
                if (!included) {
                    __DEV__ && invariant$2.warn("Unknown query ".concat(typeof nameOrDoc === "string" ? "named " : "").concat(JSON.stringify(nameOrDoc, null, 2), " requested in refetchQueries options.include array"));
                }
            });
        }
        return queries;
    };
    QueryManager.prototype.reFetchObservableQueries = function (includeStandby) {
        var _this = this;
        if (includeStandby === void 0) { includeStandby = false; }
        var observableQueryPromises = [];
        this.getObservableQueries(includeStandby ? "all" : "active").forEach(function (observableQuery, queryId) {
            var fetchPolicy = observableQuery.options.fetchPolicy;
            observableQuery.resetLastResults();
            if (includeStandby ||
                (fetchPolicy !== "standby" &&
                    fetchPolicy !== "cache-only")) {
                observableQueryPromises.push(observableQuery.refetch());
            }
            _this.getQuery(queryId).setDiff(null);
        });
        this.broadcastQueries();
        return Promise.all(observableQueryPromises);
    };
    QueryManager.prototype.setObservableQuery = function (observableQuery) {
        this.getQuery(observableQuery.queryId).setObservableQuery(observableQuery);
    };
    QueryManager.prototype.startGraphQLSubscription = function (_a) {
        var _this = this;
        var query = _a.query, fetchPolicy = _a.fetchPolicy, errorPolicy = _a.errorPolicy, variables = _a.variables, _b = _a.context, context = _b === void 0 ? {} : _b;
        query = this.transform(query).document;
        variables = this.getVariables(query, variables);
        var makeObservable = function (variables) {
            return _this.getObservableFromLink(query, context, variables).map(function (result) {
                if (fetchPolicy !== 'no-cache') {
                    if (shouldWriteResult(result, errorPolicy)) {
                        _this.cache.write({
                            query: query,
                            result: result.data,
                            dataId: 'ROOT_SUBSCRIPTION',
                            variables: variables,
                        });
                    }
                    _this.broadcastQueries();
                }
                if (graphQLResultHasError(result)) {
                    throw new ApolloError({
                        graphQLErrors: result.errors,
                    });
                }
                return result;
            });
        };
        if (this.transform(query).hasClientExports) {
            var observablePromise_1 = this.localState.addExportedVariables(query, variables, context).then(makeObservable);
            return new Observable(function (observer) {
                var sub = null;
                observablePromise_1.then(function (observable) { return sub = observable.subscribe(observer); }, observer.error);
                return function () { return sub && sub.unsubscribe(); };
            });
        }
        return makeObservable(variables);
    };
    QueryManager.prototype.stopQuery = function (queryId) {
        this.stopQueryNoBroadcast(queryId);
        this.broadcastQueries();
    };
    QueryManager.prototype.stopQueryNoBroadcast = function (queryId) {
        this.stopQueryInStoreNoBroadcast(queryId);
        this.removeQuery(queryId);
    };
    QueryManager.prototype.removeQuery = function (queryId) {
        this.fetchCancelFns.delete(queryId);
        if (this.queries.has(queryId)) {
            this.getQuery(queryId).stop();
            this.queries.delete(queryId);
        }
    };
    QueryManager.prototype.broadcastQueries = function () {
        if (this.onBroadcast)
            this.onBroadcast();
        this.queries.forEach(function (info) { return info.notify(); });
    };
    QueryManager.prototype.getLocalState = function () {
        return this.localState;
    };
    QueryManager.prototype.getObservableFromLink = function (query, context, variables, deduplication) {
        var _this = this;
        var _a;
        if (deduplication === void 0) { deduplication = (_a = context === null || context === void 0 ? void 0 : context.queryDeduplication) !== null && _a !== void 0 ? _a : this.queryDeduplication; }
        var observable;
        var serverQuery = this.transform(query).serverQuery;
        if (serverQuery) {
            var _b = this, inFlightLinkObservables_1 = _b.inFlightLinkObservables, link = _b.link;
            var operation = {
                query: serverQuery,
                variables: variables,
                operationName: getOperationName(serverQuery) || void 0,
                context: this.prepareContext(__assign$2(__assign$2({}, context), { forceFetch: !deduplication })),
            };
            context = operation.context;
            if (deduplication) {
                var byVariables_1 = inFlightLinkObservables_1.get(serverQuery) || new Map();
                inFlightLinkObservables_1.set(serverQuery, byVariables_1);
                var varJson_1 = canonicalStringify(variables);
                observable = byVariables_1.get(varJson_1);
                if (!observable) {
                    var concast = new Concast([
                        execute(link, operation)
                    ]);
                    byVariables_1.set(varJson_1, observable = concast);
                    concast.beforeNext(function () {
                        if (byVariables_1.delete(varJson_1) &&
                            byVariables_1.size < 1) {
                            inFlightLinkObservables_1.delete(serverQuery);
                        }
                    });
                }
            }
            else {
                observable = new Concast([
                    execute(link, operation)
                ]);
            }
        }
        else {
            observable = new Concast([
                Observable.of({ data: {} })
            ]);
            context = this.prepareContext(context);
        }
        var clientQuery = this.transform(query).clientQuery;
        if (clientQuery) {
            observable = asyncMap(observable, function (result) {
                return _this.localState.runResolvers({
                    document: clientQuery,
                    remoteResult: result,
                    context: context,
                    variables: variables,
                });
            });
        }
        return observable;
    };
    QueryManager.prototype.getResultsFromLink = function (queryInfo, cacheWriteBehavior, options) {
        var requestId = queryInfo.lastRequestId = this.generateRequestId();
        var linkDocument = this.cache.transformForLink(this.transform(queryInfo.document).document);
        return asyncMap(this.getObservableFromLink(linkDocument, options.context, options.variables), function (result) {
            var graphQLErrors = getGraphQLErrorsFromResult(result);
            var hasErrors = graphQLErrors.length > 0;
            if (requestId >= queryInfo.lastRequestId) {
                if (hasErrors && options.errorPolicy === "none") {
                    throw queryInfo.markError(new ApolloError({
                        graphQLErrors: graphQLErrors,
                    }));
                }
                queryInfo.markResult(result, linkDocument, options, cacheWriteBehavior);
                queryInfo.markReady();
            }
            var aqr = {
                data: result.data,
                loading: false,
                networkStatus: NetworkStatus.ready,
            };
            if (hasErrors && options.errorPolicy !== "ignore") {
                aqr.errors = graphQLErrors;
                aqr.networkStatus = NetworkStatus.error;
            }
            return aqr;
        }, function (networkError) {
            var error = isApolloError(networkError)
                ? networkError
                : new ApolloError({ networkError: networkError });
            if (requestId >= queryInfo.lastRequestId) {
                queryInfo.markError(error);
            }
            throw error;
        });
    };
    QueryManager.prototype.fetchQueryObservable = function (queryId, options, networkStatus) {
        var _this = this;
        if (networkStatus === void 0) { networkStatus = NetworkStatus.loading; }
        var query = this.transform(options.query).document;
        var variables = this.getVariables(query, options.variables);
        var queryInfo = this.getQuery(queryId);
        var defaults = this.defaultOptions.watchQuery;
        var _a = options.fetchPolicy, fetchPolicy = _a === void 0 ? defaults && defaults.fetchPolicy || "cache-first" : _a, _b = options.errorPolicy, errorPolicy = _b === void 0 ? defaults && defaults.errorPolicy || "none" : _b, _c = options.returnPartialData, returnPartialData = _c === void 0 ? false : _c, _d = options.notifyOnNetworkStatusChange, notifyOnNetworkStatusChange = _d === void 0 ? false : _d, _e = options.context, context = _e === void 0 ? {} : _e;
        var normalized = Object.assign({}, options, {
            query: query,
            variables: variables,
            fetchPolicy: fetchPolicy,
            errorPolicy: errorPolicy,
            returnPartialData: returnPartialData,
            notifyOnNetworkStatusChange: notifyOnNetworkStatusChange,
            context: context,
        });
        var fromVariables = function (variables) {
            normalized.variables = variables;
            var concastSources = _this.fetchQueryByPolicy(queryInfo, normalized, networkStatus);
            if (normalized.fetchPolicy !== "standby" &&
                concastSources.length > 0 &&
                queryInfo.observableQuery) {
                queryInfo.observableQuery["applyNextFetchPolicy"]("after-fetch", options);
            }
            return concastSources;
        };
        var cleanupCancelFn = function () { return _this.fetchCancelFns.delete(queryId); };
        this.fetchCancelFns.set(queryId, function (reason) {
            cleanupCancelFn();
            setTimeout(function () { return concast.cancel(reason); });
        });
        var concast = new Concast(this.transform(normalized.query).hasClientExports
            ? this.localState.addExportedVariables(normalized.query, normalized.variables, normalized.context).then(fromVariables)
            : fromVariables(normalized.variables));
        concast.promise.then(cleanupCancelFn, cleanupCancelFn);
        return concast;
    };
    QueryManager.prototype.refetchQueries = function (_a) {
        var _this = this;
        var updateCache = _a.updateCache, include = _a.include, _b = _a.optimistic, optimistic = _b === void 0 ? false : _b, _c = _a.removeOptimistic, removeOptimistic = _c === void 0 ? optimistic ? makeUniqueId("refetchQueries") : void 0 : _c, onQueryUpdated = _a.onQueryUpdated;
        var includedQueriesById = new Map();
        if (include) {
            this.getObservableQueries(include).forEach(function (oq, queryId) {
                includedQueriesById.set(queryId, {
                    oq: oq,
                    lastDiff: _this.getQuery(queryId).getDiff(),
                });
            });
        }
        var results = new Map;
        if (updateCache) {
            this.cache.batch({
                update: updateCache,
                optimistic: optimistic && removeOptimistic || false,
                removeOptimistic: removeOptimistic,
                onWatchUpdated: function (watch, diff, lastDiff) {
                    var oq = watch.watcher instanceof QueryInfo &&
                        watch.watcher.observableQuery;
                    if (oq) {
                        if (onQueryUpdated) {
                            includedQueriesById.delete(oq.queryId);
                            var result = onQueryUpdated(oq, diff, lastDiff);
                            if (result === true) {
                                result = oq.refetch();
                            }
                            if (result !== false) {
                                results.set(oq, result);
                            }
                            return result;
                        }
                        if (onQueryUpdated !== null) {
                            includedQueriesById.set(oq.queryId, { oq: oq, lastDiff: lastDiff, diff: diff });
                        }
                    }
                },
            });
        }
        if (includedQueriesById.size) {
            includedQueriesById.forEach(function (_a, queryId) {
                var oq = _a.oq, lastDiff = _a.lastDiff, diff = _a.diff;
                var result;
                if (onQueryUpdated) {
                    if (!diff) {
                        var info = oq["queryInfo"];
                        info.reset();
                        diff = info.getDiff();
                    }
                    result = onQueryUpdated(oq, diff, lastDiff);
                }
                if (!onQueryUpdated || result === true) {
                    result = oq.refetch();
                }
                if (result !== false) {
                    results.set(oq, result);
                }
                if (queryId.indexOf("legacyOneTimeQuery") >= 0) {
                    _this.stopQueryNoBroadcast(queryId);
                }
            });
        }
        if (removeOptimistic) {
            this.cache.removeOptimistic(removeOptimistic);
        }
        return results;
    };
    QueryManager.prototype.fetchQueryByPolicy = function (queryInfo, _a, networkStatus) {
        var _this = this;
        var query = _a.query, variables = _a.variables, fetchPolicy = _a.fetchPolicy, refetchWritePolicy = _a.refetchWritePolicy, errorPolicy = _a.errorPolicy, returnPartialData = _a.returnPartialData, context = _a.context, notifyOnNetworkStatusChange = _a.notifyOnNetworkStatusChange;
        var oldNetworkStatus = queryInfo.networkStatus;
        queryInfo.init({
            document: this.transform(query).document,
            variables: variables,
            networkStatus: networkStatus,
        });
        var readCache = function () { return queryInfo.getDiff(variables); };
        var resultsFromCache = function (diff, networkStatus) {
            if (networkStatus === void 0) { networkStatus = queryInfo.networkStatus || NetworkStatus.loading; }
            var data = diff.result;
            if (__DEV__ &&
                !returnPartialData &&
                !equal(data, {})) {
                logMissingFieldErrors(diff.missing);
            }
            var fromData = function (data) { return Observable.of(__assign$2({ data: data, loading: isNetworkRequestInFlight(networkStatus), networkStatus: networkStatus }, (diff.complete ? null : { partial: true }))); };
            if (data && _this.transform(query).hasForcedResolvers) {
                return _this.localState.runResolvers({
                    document: query,
                    remoteResult: { data: data },
                    context: context,
                    variables: variables,
                    onlyRunForcedResolvers: true,
                }).then(function (resolved) { return fromData(resolved.data || void 0); });
            }
            if (errorPolicy === 'none' &&
                networkStatus === NetworkStatus.refetch &&
                Array.isArray(diff.missing)) {
                return fromData(void 0);
            }
            return fromData(data);
        };
        var cacheWriteBehavior = fetchPolicy === "no-cache" ? 0 :
            (networkStatus === NetworkStatus.refetch &&
                refetchWritePolicy !== "merge") ? 1
                : 2;
        var resultsFromLink = function () { return _this.getResultsFromLink(queryInfo, cacheWriteBehavior, {
            variables: variables,
            context: context,
            fetchPolicy: fetchPolicy,
            errorPolicy: errorPolicy,
        }); };
        var shouldNotify = notifyOnNetworkStatusChange &&
            typeof oldNetworkStatus === "number" &&
            oldNetworkStatus !== networkStatus &&
            isNetworkRequestInFlight(networkStatus);
        switch (fetchPolicy) {
            default:
            case "cache-first": {
                var diff = readCache();
                if (diff.complete) {
                    return [
                        resultsFromCache(diff, queryInfo.markReady()),
                    ];
                }
                if (returnPartialData || shouldNotify) {
                    return [
                        resultsFromCache(diff),
                        resultsFromLink(),
                    ];
                }
                return [
                    resultsFromLink(),
                ];
            }
            case "cache-and-network": {
                var diff = readCache();
                if (diff.complete || returnPartialData || shouldNotify) {
                    return [
                        resultsFromCache(diff),
                        resultsFromLink(),
                    ];
                }
                return [
                    resultsFromLink(),
                ];
            }
            case "cache-only":
                return [
                    resultsFromCache(readCache(), queryInfo.markReady()),
                ];
            case "network-only":
                if (shouldNotify) {
                    return [
                        resultsFromCache(readCache()),
                        resultsFromLink(),
                    ];
                }
                return [resultsFromLink()];
            case "no-cache":
                if (shouldNotify) {
                    return [
                        resultsFromCache(queryInfo.getDiff()),
                        resultsFromLink(),
                    ];
                }
                return [resultsFromLink()];
            case "standby":
                return [];
        }
    };
    QueryManager.prototype.getQuery = function (queryId) {
        if (queryId && !this.queries.has(queryId)) {
            this.queries.set(queryId, new QueryInfo(this, queryId));
        }
        return this.queries.get(queryId);
    };
    QueryManager.prototype.prepareContext = function (context) {
        if (context === void 0) { context = {}; }
        var newContext = this.localState.prepareContext(context);
        return __assign$2(__assign$2({}, newContext), { clientAwareness: this.clientAwareness });
    };
    return QueryManager;
}());

var hasSuggestedDevtools = false;
var ApolloClient = (function () {
    function ApolloClient(options) {
        var _this = this;
        this.resetStoreCallbacks = [];
        this.clearStoreCallbacks = [];
        var uri = options.uri, credentials = options.credentials, headers = options.headers, cache = options.cache, _a = options.ssrMode, ssrMode = _a === void 0 ? false : _a, _b = options.ssrForceFetchDelay, ssrForceFetchDelay = _b === void 0 ? 0 : _b, _c = options.connectToDevTools, connectToDevTools = _c === void 0 ? typeof window === 'object' &&
            !window.__APOLLO_CLIENT__ &&
            __DEV__ : _c, _d = options.queryDeduplication, queryDeduplication = _d === void 0 ? true : _d, defaultOptions = options.defaultOptions, _e = options.assumeImmutableResults, assumeImmutableResults = _e === void 0 ? false : _e, resolvers = options.resolvers, typeDefs = options.typeDefs, fragmentMatcher = options.fragmentMatcher, clientAwarenessName = options.name, clientAwarenessVersion = options.version;
        var link = options.link;
        if (!link) {
            link = uri
                ? new HttpLink({ uri: uri, credentials: credentials, headers: headers })
                : ApolloLink.empty();
        }
        if (!cache) {
            throw __DEV__ ? new InvariantError("To initialize Apollo Client, you must specify a 'cache' property " +
                "in the options object. \n" +
                "For more information, please visit: https://go.apollo.dev/c/docs") : new InvariantError(9);
        }
        this.link = link;
        this.cache = cache;
        this.disableNetworkFetches = ssrMode || ssrForceFetchDelay > 0;
        this.queryDeduplication = queryDeduplication;
        this.defaultOptions = defaultOptions || Object.create(null);
        this.typeDefs = typeDefs;
        if (ssrForceFetchDelay) {
            setTimeout(function () { return (_this.disableNetworkFetches = false); }, ssrForceFetchDelay);
        }
        this.watchQuery = this.watchQuery.bind(this);
        this.query = this.query.bind(this);
        this.mutate = this.mutate.bind(this);
        this.resetStore = this.resetStore.bind(this);
        this.reFetchObservableQueries = this.reFetchObservableQueries.bind(this);
        if (connectToDevTools && typeof window === 'object') {
            window.__APOLLO_CLIENT__ = this;
        }
        if (!hasSuggestedDevtools && connectToDevTools && __DEV__) {
            hasSuggestedDevtools = true;
            if (typeof window !== 'undefined' &&
                window.document &&
                window.top === window.self &&
                !window.__APOLLO_DEVTOOLS_GLOBAL_HOOK__) {
                var nav = window.navigator;
                var ua = nav && nav.userAgent;
                var url = void 0;
                if (typeof ua === "string") {
                    if (ua.indexOf("Chrome/") > -1) {
                        url = "https://chrome.google.com/webstore/detail/" +
                            "apollo-client-developer-t/jdkknkkbebbapilgoeccciglkfbmbnfm";
                    }
                    else if (ua.indexOf("Firefox/") > -1) {
                        url = "https://addons.mozilla.org/en-US/firefox/addon/apollo-developer-tools/";
                    }
                }
                if (url) {
                    __DEV__ && invariant$2.log("Download the Apollo DevTools for a better development " +
                        "experience: " + url);
                }
            }
        }
        this.version = version;
        this.localState = new LocalState({
            cache: cache,
            client: this,
            resolvers: resolvers,
            fragmentMatcher: fragmentMatcher,
        });
        this.queryManager = new QueryManager({
            cache: this.cache,
            link: this.link,
            defaultOptions: this.defaultOptions,
            queryDeduplication: queryDeduplication,
            ssrMode: ssrMode,
            clientAwareness: {
                name: clientAwarenessName,
                version: clientAwarenessVersion,
            },
            localState: this.localState,
            assumeImmutableResults: assumeImmutableResults,
            onBroadcast: connectToDevTools ? function () {
                if (_this.devToolsHookCb) {
                    _this.devToolsHookCb({
                        action: {},
                        state: {
                            queries: _this.queryManager.getQueryStore(),
                            mutations: _this.queryManager.mutationStore || {},
                        },
                        dataWithOptimisticResults: _this.cache.extract(true),
                    });
                }
            } : void 0,
        });
    }
    ApolloClient.prototype.stop = function () {
        this.queryManager.stop();
    };
    ApolloClient.prototype.watchQuery = function (options) {
        if (this.defaultOptions.watchQuery) {
            options = mergeOptions(this.defaultOptions.watchQuery, options);
        }
        if (this.disableNetworkFetches &&
            (options.fetchPolicy === 'network-only' ||
                options.fetchPolicy === 'cache-and-network')) {
            options = __assign$2(__assign$2({}, options), { fetchPolicy: 'cache-first' });
        }
        return this.queryManager.watchQuery(options);
    };
    ApolloClient.prototype.query = function (options) {
        if (this.defaultOptions.query) {
            options = mergeOptions(this.defaultOptions.query, options);
        }
        __DEV__ ? invariant$2(options.fetchPolicy !== 'cache-and-network', 'The cache-and-network fetchPolicy does not work with client.query, because ' +
            'client.query can only return a single result. Please use client.watchQuery ' +
            'to receive multiple results from the cache and the network, or consider ' +
            'using a different fetchPolicy, such as cache-first or network-only.') : invariant$2(options.fetchPolicy !== 'cache-and-network', 10);
        if (this.disableNetworkFetches && options.fetchPolicy === 'network-only') {
            options = __assign$2(__assign$2({}, options), { fetchPolicy: 'cache-first' });
        }
        return this.queryManager.query(options);
    };
    ApolloClient.prototype.mutate = function (options) {
        if (this.defaultOptions.mutate) {
            options = mergeOptions(this.defaultOptions.mutate, options);
        }
        return this.queryManager.mutate(options);
    };
    ApolloClient.prototype.subscribe = function (options) {
        return this.queryManager.startGraphQLSubscription(options);
    };
    ApolloClient.prototype.readQuery = function (options, optimistic) {
        if (optimistic === void 0) { optimistic = false; }
        return this.cache.readQuery(options, optimistic);
    };
    ApolloClient.prototype.readFragment = function (options, optimistic) {
        if (optimistic === void 0) { optimistic = false; }
        return this.cache.readFragment(options, optimistic);
    };
    ApolloClient.prototype.writeQuery = function (options) {
        var ref = this.cache.writeQuery(options);
        if (options.broadcast !== false) {
            this.queryManager.broadcastQueries();
        }
        return ref;
    };
    ApolloClient.prototype.writeFragment = function (options) {
        var ref = this.cache.writeFragment(options);
        if (options.broadcast !== false) {
            this.queryManager.broadcastQueries();
        }
        return ref;
    };
    ApolloClient.prototype.__actionHookForDevTools = function (cb) {
        this.devToolsHookCb = cb;
    };
    ApolloClient.prototype.__requestRaw = function (payload) {
        return execute(this.link, payload);
    };
    ApolloClient.prototype.resetStore = function () {
        var _this = this;
        return Promise.resolve()
            .then(function () { return _this.queryManager.clearStore({
            discardWatches: false,
        }); })
            .then(function () { return Promise.all(_this.resetStoreCallbacks.map(function (fn) { return fn(); })); })
            .then(function () { return _this.reFetchObservableQueries(); });
    };
    ApolloClient.prototype.clearStore = function () {
        var _this = this;
        return Promise.resolve()
            .then(function () { return _this.queryManager.clearStore({
            discardWatches: true,
        }); })
            .then(function () { return Promise.all(_this.clearStoreCallbacks.map(function (fn) { return fn(); })); });
    };
    ApolloClient.prototype.onResetStore = function (cb) {
        var _this = this;
        this.resetStoreCallbacks.push(cb);
        return function () {
            _this.resetStoreCallbacks = _this.resetStoreCallbacks.filter(function (c) { return c !== cb; });
        };
    };
    ApolloClient.prototype.onClearStore = function (cb) {
        var _this = this;
        this.clearStoreCallbacks.push(cb);
        return function () {
            _this.clearStoreCallbacks = _this.clearStoreCallbacks.filter(function (c) { return c !== cb; });
        };
    };
    ApolloClient.prototype.reFetchObservableQueries = function (includeStandby) {
        return this.queryManager.reFetchObservableQueries(includeStandby);
    };
    ApolloClient.prototype.refetchQueries = function (options) {
        var map = this.queryManager.refetchQueries(options);
        var queries = [];
        var results = [];
        map.forEach(function (result, obsQuery) {
            queries.push(obsQuery);
            results.push(result);
        });
        var result = Promise.all(results);
        result.queries = queries;
        result.results = results;
        result.catch(function (error) {
            __DEV__ && invariant$2.debug("In client.refetchQueries, Promise.all promise rejected with error ".concat(error));
        });
        return result;
    };
    ApolloClient.prototype.getObservableQueries = function (include) {
        if (include === void 0) { include = "active"; }
        return this.queryManager.getObservableQueries(include);
    };
    ApolloClient.prototype.extract = function (optimistic) {
        return this.cache.extract(optimistic);
    };
    ApolloClient.prototype.restore = function (serializedState) {
        return this.cache.restore(serializedState);
    };
    ApolloClient.prototype.addResolvers = function (resolvers) {
        this.localState.addResolvers(resolvers);
    };
    ApolloClient.prototype.setResolvers = function (resolvers) {
        this.localState.setResolvers(resolvers);
    };
    ApolloClient.prototype.getResolvers = function () {
        return this.localState.getResolvers();
    };
    ApolloClient.prototype.setLocalStateFragmentMatcher = function (fragmentMatcher) {
        this.localState.setFragmentMatcher(fragmentMatcher);
    };
    ApolloClient.prototype.setLink = function (newLink) {
        this.link = this.queryManager.link = newLink;
    };
    return ApolloClient;
}());

var commonjsGlobal = typeof globalThis !== 'undefined' ? globalThis : typeof window !== 'undefined' ? window : typeof global !== 'undefined' ? global : typeof self !== 'undefined' ? self : {};

function getAugmentedNamespace(n) {
  if (n.__esModule) return n;
  var f = n.default;
	if (typeof f == "function") {
		var a = function a () {
			if (this instanceof a) {
        return Reflect.construct(f, arguments, this.constructor);
			}
			return f.apply(this, arguments);
		};
		a.prototype = f.prototype;
  } else a = {};
  Object.defineProperty(a, '__esModule', {value: true});
	Object.keys(n).forEach(function (k) {
		var d = Object.getOwnPropertyDescriptor(n, k);
		Object.defineProperty(a, k, d.get ? d : {
			enumerable: true,
			get: function () {
				return n[k];
			}
		});
	});
	return a;
}

var client = {};

var eventemitter3 = {exports: {}};

(function (module) {

	var has = Object.prototype.hasOwnProperty
	  , prefix = '~';

	/**
	 * Constructor to create a storage for our `EE` objects.
	 * An `Events` instance is a plain object whose properties are event names.
	 *
	 * @constructor
	 * @private
	 */
	function Events() {}

	//
	// We try to not inherit from `Object.prototype`. In some engines creating an
	// instance in this way is faster than calling `Object.create(null)` directly.
	// If `Object.create(null)` is not supported we prefix the event names with a
	// character to make sure that the built-in object properties are not
	// overridden or used as an attack vector.
	//
	if (Object.create) {
	  Events.prototype = Object.create(null);

	  //
	  // This hack is needed because the `__proto__` property is still inherited in
	  // some old browsers like Android 4, iPhone 5.1, Opera 11 and Safari 5.
	  //
	  if (!new Events().__proto__) prefix = false;
	}

	/**
	 * Representation of a single event listener.
	 *
	 * @param {Function} fn The listener function.
	 * @param {*} context The context to invoke the listener with.
	 * @param {Boolean} [once=false] Specify if the listener is a one-time listener.
	 * @constructor
	 * @private
	 */
	function EE(fn, context, once) {
	  this.fn = fn;
	  this.context = context;
	  this.once = once || false;
	}

	/**
	 * Add a listener for a given event.
	 *
	 * @param {EventEmitter} emitter Reference to the `EventEmitter` instance.
	 * @param {(String|Symbol)} event The event name.
	 * @param {Function} fn The listener function.
	 * @param {*} context The context to invoke the listener with.
	 * @param {Boolean} once Specify if the listener is a one-time listener.
	 * @returns {EventEmitter}
	 * @private
	 */
	function addListener(emitter, event, fn, context, once) {
	  if (typeof fn !== 'function') {
	    throw new TypeError('The listener must be a function');
	  }

	  var listener = new EE(fn, context || emitter, once)
	    , evt = prefix ? prefix + event : event;

	  if (!emitter._events[evt]) emitter._events[evt] = listener, emitter._eventsCount++;
	  else if (!emitter._events[evt].fn) emitter._events[evt].push(listener);
	  else emitter._events[evt] = [emitter._events[evt], listener];

	  return emitter;
	}

	/**
	 * Clear event by name.
	 *
	 * @param {EventEmitter} emitter Reference to the `EventEmitter` instance.
	 * @param {(String|Symbol)} evt The Event name.
	 * @private
	 */
	function clearEvent(emitter, evt) {
	  if (--emitter._eventsCount === 0) emitter._events = new Events();
	  else delete emitter._events[evt];
	}

	/**
	 * Minimal `EventEmitter` interface that is molded against the Node.js
	 * `EventEmitter` interface.
	 *
	 * @constructor
	 * @public
	 */
	function EventEmitter() {
	  this._events = new Events();
	  this._eventsCount = 0;
	}

	/**
	 * Return an array listing the events for which the emitter has registered
	 * listeners.
	 *
	 * @returns {Array}
	 * @public
	 */
	EventEmitter.prototype.eventNames = function eventNames() {
	  var names = []
	    , events
	    , name;

	  if (this._eventsCount === 0) return names;

	  for (name in (events = this._events)) {
	    if (has.call(events, name)) names.push(prefix ? name.slice(1) : name);
	  }

	  if (Object.getOwnPropertySymbols) {
	    return names.concat(Object.getOwnPropertySymbols(events));
	  }

	  return names;
	};

	/**
	 * Return the listeners registered for a given event.
	 *
	 * @param {(String|Symbol)} event The event name.
	 * @returns {Array} The registered listeners.
	 * @public
	 */
	EventEmitter.prototype.listeners = function listeners(event) {
	  var evt = prefix ? prefix + event : event
	    , handlers = this._events[evt];

	  if (!handlers) return [];
	  if (handlers.fn) return [handlers.fn];

	  for (var i = 0, l = handlers.length, ee = new Array(l); i < l; i++) {
	    ee[i] = handlers[i].fn;
	  }

	  return ee;
	};

	/**
	 * Return the number of listeners listening to a given event.
	 *
	 * @param {(String|Symbol)} event The event name.
	 * @returns {Number} The number of listeners.
	 * @public
	 */
	EventEmitter.prototype.listenerCount = function listenerCount(event) {
	  var evt = prefix ? prefix + event : event
	    , listeners = this._events[evt];

	  if (!listeners) return 0;
	  if (listeners.fn) return 1;
	  return listeners.length;
	};

	/**
	 * Calls each of the listeners registered for a given event.
	 *
	 * @param {(String|Symbol)} event The event name.
	 * @returns {Boolean} `true` if the event had listeners, else `false`.
	 * @public
	 */
	EventEmitter.prototype.emit = function emit(event, a1, a2, a3, a4, a5) {
	  var evt = prefix ? prefix + event : event;

	  if (!this._events[evt]) return false;

	  var listeners = this._events[evt]
	    , len = arguments.length
	    , args
	    , i;

	  if (listeners.fn) {
	    if (listeners.once) this.removeListener(event, listeners.fn, undefined, true);

	    switch (len) {
	      case 1: return listeners.fn.call(listeners.context), true;
	      case 2: return listeners.fn.call(listeners.context, a1), true;
	      case 3: return listeners.fn.call(listeners.context, a1, a2), true;
	      case 4: return listeners.fn.call(listeners.context, a1, a2, a3), true;
	      case 5: return listeners.fn.call(listeners.context, a1, a2, a3, a4), true;
	      case 6: return listeners.fn.call(listeners.context, a1, a2, a3, a4, a5), true;
	    }

	    for (i = 1, args = new Array(len -1); i < len; i++) {
	      args[i - 1] = arguments[i];
	    }

	    listeners.fn.apply(listeners.context, args);
	  } else {
	    var length = listeners.length
	      , j;

	    for (i = 0; i < length; i++) {
	      if (listeners[i].once) this.removeListener(event, listeners[i].fn, undefined, true);

	      switch (len) {
	        case 1: listeners[i].fn.call(listeners[i].context); break;
	        case 2: listeners[i].fn.call(listeners[i].context, a1); break;
	        case 3: listeners[i].fn.call(listeners[i].context, a1, a2); break;
	        case 4: listeners[i].fn.call(listeners[i].context, a1, a2, a3); break;
	        default:
	          if (!args) for (j = 1, args = new Array(len -1); j < len; j++) {
	            args[j - 1] = arguments[j];
	          }

	          listeners[i].fn.apply(listeners[i].context, args);
	      }
	    }
	  }

	  return true;
	};

	/**
	 * Add a listener for a given event.
	 *
	 * @param {(String|Symbol)} event The event name.
	 * @param {Function} fn The listener function.
	 * @param {*} [context=this] The context to invoke the listener with.
	 * @returns {EventEmitter} `this`.
	 * @public
	 */
	EventEmitter.prototype.on = function on(event, fn, context) {
	  return addListener(this, event, fn, context, false);
	};

	/**
	 * Add a one-time listener for a given event.
	 *
	 * @param {(String|Symbol)} event The event name.
	 * @param {Function} fn The listener function.
	 * @param {*} [context=this] The context to invoke the listener with.
	 * @returns {EventEmitter} `this`.
	 * @public
	 */
	EventEmitter.prototype.once = function once(event, fn, context) {
	  return addListener(this, event, fn, context, true);
	};

	/**
	 * Remove the listeners of a given event.
	 *
	 * @param {(String|Symbol)} event The event name.
	 * @param {Function} fn Only remove the listeners that match this function.
	 * @param {*} context Only remove the listeners that have this context.
	 * @param {Boolean} once Only remove one-time listeners.
	 * @returns {EventEmitter} `this`.
	 * @public
	 */
	EventEmitter.prototype.removeListener = function removeListener(event, fn, context, once) {
	  var evt = prefix ? prefix + event : event;

	  if (!this._events[evt]) return this;
	  if (!fn) {
	    clearEvent(this, evt);
	    return this;
	  }

	  var listeners = this._events[evt];

	  if (listeners.fn) {
	    if (
	      listeners.fn === fn &&
	      (!once || listeners.once) &&
	      (!context || listeners.context === context)
	    ) {
	      clearEvent(this, evt);
	    }
	  } else {
	    for (var i = 0, events = [], length = listeners.length; i < length; i++) {
	      if (
	        listeners[i].fn !== fn ||
	        (once && !listeners[i].once) ||
	        (context && listeners[i].context !== context)
	      ) {
	        events.push(listeners[i]);
	      }
	    }

	    //
	    // Reset the array, or remove it completely if we have no more listeners.
	    //
	    if (events.length) this._events[evt] = events.length === 1 ? events[0] : events;
	    else clearEvent(this, evt);
	  }

	  return this;
	};

	/**
	 * Remove all listeners, or those of the specified event.
	 *
	 * @param {(String|Symbol)} [event] The event name.
	 * @returns {EventEmitter} `this`.
	 * @public
	 */
	EventEmitter.prototype.removeAllListeners = function removeAllListeners(event) {
	  var evt;

	  if (event) {
	    evt = prefix ? prefix + event : event;
	    if (this._events[evt]) clearEvent(this, evt);
	  } else {
	    this._events = new Events();
	    this._eventsCount = 0;
	  }

	  return this;
	};

	//
	// Alias methods names because people roll like that.
	//
	EventEmitter.prototype.off = EventEmitter.prototype.removeListener;
	EventEmitter.prototype.addListener = EventEmitter.prototype.on;

	//
	// Expose the prefix.
	//
	EventEmitter.prefixed = prefix;

	//
	// Allow `EventEmitter` to be imported as module namespace.
	//
	EventEmitter.EventEmitter = EventEmitter;

	//
	// Expose the module.
	//
	{
	  module.exports = EventEmitter;
	} 
} (eventemitter3));

var eventemitter3Exports = eventemitter3.exports;

var isString$1 = {};

Object.defineProperty(isString$1, "__esModule", { value: true });
function isString(value) {
    return typeof value === 'string';
}
isString$1.default = isString;

var isObject$1 = {};

Object.defineProperty(isObject$1, "__esModule", { value: true });
function isObject(value) {
    return ((value !== null) && (typeof value === 'object'));
}
isObject$1.default = isObject;

function symbolObservablePonyfill(root) {
	var result;
	var Symbol = root.Symbol;

	if (typeof Symbol === 'function') {
		if (Symbol.observable) {
			result = Symbol.observable;
		} else {
			result = Symbol('observable');
			Symbol.observable = result;
		}
	} else {
		result = '@@observable';
	}

	return result;
}

/* global window */

var root;

if (typeof self !== 'undefined') {
  root = self;
} else if (typeof window !== 'undefined') {
  root = window;
} else if (typeof global !== 'undefined') {
  root = global;
} else if (typeof module !== 'undefined') {
  root = module;
} else {
  root = Function('return this')();
}

var result = symbolObservablePonyfill(root);

var es = /*#__PURE__*/Object.freeze({
	__proto__: null,
	default: result
});

var require$$6 = /*@__PURE__*/getAugmentedNamespace(es);

var protocol = {};

Object.defineProperty(protocol, "__esModule", { value: true });
protocol.GRAPHQL_SUBSCRIPTIONS = protocol.GRAPHQL_WS = void 0;
var GRAPHQL_WS = 'graphql-ws';
protocol.GRAPHQL_WS = GRAPHQL_WS;
var GRAPHQL_SUBSCRIPTIONS = 'graphql-subscriptions';
protocol.GRAPHQL_SUBSCRIPTIONS = GRAPHQL_SUBSCRIPTIONS;

var defaults = {};

Object.defineProperty(defaults, "__esModule", { value: true });
defaults.WS_TIMEOUT = defaults.MIN_WS_TIMEOUT = void 0;
var MIN_WS_TIMEOUT = 1000;
defaults.MIN_WS_TIMEOUT = MIN_WS_TIMEOUT;
var WS_TIMEOUT = 30000;
defaults.WS_TIMEOUT = WS_TIMEOUT;

var messageTypes = {};

Object.defineProperty(messageTypes, "__esModule", { value: true });
var MessageTypes = (function () {
    function MessageTypes() {
        throw new Error('Static Class');
    }
    MessageTypes.GQL_CONNECTION_INIT = 'connection_init';
    MessageTypes.GQL_CONNECTION_ACK = 'connection_ack';
    MessageTypes.GQL_CONNECTION_ERROR = 'connection_error';
    MessageTypes.GQL_CONNECTION_KEEP_ALIVE = 'ka';
    MessageTypes.GQL_CONNECTION_TERMINATE = 'connection_terminate';
    MessageTypes.GQL_START = 'start';
    MessageTypes.GQL_DATA = 'data';
    MessageTypes.GQL_ERROR = 'error';
    MessageTypes.GQL_COMPLETE = 'complete';
    MessageTypes.GQL_STOP = 'stop';
    MessageTypes.SUBSCRIPTION_START = 'subscription_start';
    MessageTypes.SUBSCRIPTION_DATA = 'subscription_data';
    MessageTypes.SUBSCRIPTION_SUCCESS = 'subscription_success';
    MessageTypes.SUBSCRIPTION_FAIL = 'subscription_fail';
    MessageTypes.SUBSCRIPTION_END = 'subscription_end';
    MessageTypes.INIT = 'init';
    MessageTypes.INIT_SUCCESS = 'init_success';
    MessageTypes.INIT_FAIL = 'init_fail';
    MessageTypes.KEEP_ALIVE = 'keepalive';
    return MessageTypes;
}());
messageTypes.default = MessageTypes;

var __assign$1 = (commonjsGlobal && commonjsGlobal.__assign) || function () {
    __assign$1 = Object.assign || function(t) {
        for (var s, i = 1, n = arguments.length; i < n; i++) {
            s = arguments[i];
            for (var p in s) if (Object.prototype.hasOwnProperty.call(s, p))
                t[p] = s[p];
        }
        return t;
    };
    return __assign$1.apply(this, arguments);
};
var __awaiter = (commonjsGlobal && commonjsGlobal.__awaiter) || function (thisArg, _arguments, P, generator) {
    function adopt(value) { return value instanceof P ? value : new P(function (resolve) { resolve(value); }); }
    return new (P || (P = Promise))(function (resolve, reject) {
        function fulfilled(value) { try { step(generator.next(value)); } catch (e) { reject(e); } }
        function rejected(value) { try { step(generator["throw"](value)); } catch (e) { reject(e); } }
        function step(result) { result.done ? resolve(result.value) : adopt(result.value).then(fulfilled, rejected); }
        step((generator = generator.apply(thisArg, _arguments || [])).next());
    });
};
var __generator = (commonjsGlobal && commonjsGlobal.__generator) || function (thisArg, body) {
    var _ = { label: 0, sent: function() { if (t[0] & 1) throw t[1]; return t[1]; }, trys: [], ops: [] }, f, y, t, g;
    return g = { next: verb(0), "throw": verb(1), "return": verb(2) }, typeof Symbol === "function" && (g[Symbol.iterator] = function() { return this; }), g;
    function verb(n) { return function (v) { return step([n, v]); }; }
    function step(op) {
        if (f) throw new TypeError("Generator is already executing.");
        while (_) try {
            if (f = 1, y && (t = op[0] & 2 ? y["return"] : op[0] ? y["throw"] || ((t = y["return"]) && t.call(y), 0) : y.next) && !(t = t.call(y, op[1])).done) return t;
            if (y = 0, t) op = [op[0] & 2, t.value];
            switch (op[0]) {
                case 0: case 1: t = op; break;
                case 4: _.label++; return { value: op[1], done: false };
                case 5: _.label++; y = op[1]; op = [0]; continue;
                case 7: op = _.ops.pop(); _.trys.pop(); continue;
                default:
                    if (!(t = _.trys, t = t.length > 0 && t[t.length - 1]) && (op[0] === 6 || op[0] === 2)) { _ = 0; continue; }
                    if (op[0] === 3 && (!t || (op[1] > t[0] && op[1] < t[3]))) { _.label = op[1]; break; }
                    if (op[0] === 6 && _.label < t[1]) { _.label = t[1]; t = op; break; }
                    if (t && _.label < t[2]) { _.label = t[2]; _.ops.push(op); break; }
                    if (t[2]) _.ops.pop();
                    _.trys.pop(); continue;
            }
            op = body.call(thisArg, _);
        } catch (e) { op = [6, e]; y = 0; } finally { f = t = 0; }
        if (op[0] & 5) throw op[1]; return { value: op[0] ? op[1] : void 0, done: true };
    }
};
var __spreadArray = (commonjsGlobal && commonjsGlobal.__spreadArray) || function (to, from, pack) {
    if (pack || arguments.length === 2) for (var i = 0, l = from.length, ar; i < l; i++) {
        if (ar || !(i in from)) {
            if (!ar) ar = Array.prototype.slice.call(from, 0, i);
            ar[i] = from[i];
        }
    }
    return to.concat(ar || Array.prototype.slice.call(from));
};
Object.defineProperty(client, "__esModule", { value: true });
var SubscriptionClient_1 = client.SubscriptionClient = void 0;
var _global = typeof commonjsGlobal !== 'undefined' ? commonjsGlobal : (typeof window !== 'undefined' ? window : {});
var NativeWebSocket = _global.WebSocket || _global.MozWebSocket;
var Backoff = require$$0;
var eventemitter3_1 = eventemitter3Exports;
var is_string_1 = isString$1;
var is_object_1 = isObject$1;
var printer_1 = require$$4;
var getOperationAST_1 = require$$5;
var symbol_observable_1 = require$$6;
var protocol_1 = protocol;
var defaults_1 = defaults;
var message_types_1 = messageTypes;
var SubscriptionClient = (function () {
    function SubscriptionClient(url, options, webSocketImpl, webSocketProtocols) {
        var _a = (options || {}), _b = _a.connectionCallback, connectionCallback = _b === void 0 ? undefined : _b, _c = _a.connectionParams, connectionParams = _c === void 0 ? {} : _c, _d = _a.minTimeout, minTimeout = _d === void 0 ? defaults_1.MIN_WS_TIMEOUT : _d, _e = _a.timeout, timeout = _e === void 0 ? defaults_1.WS_TIMEOUT : _e, _f = _a.reconnect, reconnect = _f === void 0 ? false : _f, _g = _a.reconnectionAttempts, reconnectionAttempts = _g === void 0 ? Infinity : _g, _h = _a.lazy, lazy = _h === void 0 ? false : _h, _j = _a.inactivityTimeout, inactivityTimeout = _j === void 0 ? 0 : _j, _k = _a.wsOptionArguments, wsOptionArguments = _k === void 0 ? [] : _k;
        this.wsImpl = webSocketImpl || NativeWebSocket;
        if (!this.wsImpl) {
            throw new Error('Unable to find native implementation, or alternative implementation for WebSocket!');
        }
        this.wsProtocols = webSocketProtocols || protocol_1.GRAPHQL_WS;
        this.connectionCallback = connectionCallback;
        this.url = url;
        this.operations = {};
        this.nextOperationId = 0;
        this.minWsTimeout = minTimeout;
        this.wsTimeout = timeout;
        this.unsentMessagesQueue = [];
        this.reconnect = reconnect;
        this.reconnecting = false;
        this.reconnectionAttempts = reconnectionAttempts;
        this.lazy = !!lazy;
        this.inactivityTimeout = inactivityTimeout;
        this.closedByUser = false;
        this.backoff = new Backoff({ jitter: 0.5 });
        this.eventEmitter = new eventemitter3_1.EventEmitter();
        this.middlewares = [];
        this.client = null;
        this.maxConnectTimeGenerator = this.createMaxConnectTimeGenerator();
        this.connectionParams = this.getConnectionParams(connectionParams);
        this.wsOptionArguments = wsOptionArguments;
        if (!this.lazy) {
            this.connect();
        }
    }
    Object.defineProperty(SubscriptionClient.prototype, "status", {
        get: function () {
            if (this.client === null) {
                return this.wsImpl.CLOSED;
            }
            return this.client.readyState;
        },
        enumerable: false,
        configurable: true
    });
    SubscriptionClient.prototype.close = function (isForced, closedByUser) {
        if (isForced === void 0) { isForced = true; }
        if (closedByUser === void 0) { closedByUser = true; }
        this.clearInactivityTimeout();
        if (this.client !== null) {
            this.closedByUser = closedByUser;
            if (isForced) {
                this.clearCheckConnectionInterval();
                this.clearMaxConnectTimeout();
                this.clearTryReconnectTimeout();
                this.unsubscribeAll();
                this.sendMessage(undefined, message_types_1.default.GQL_CONNECTION_TERMINATE, null);
            }
            this.client.close();
            this.client.onopen = null;
            this.client.onclose = null;
            this.client.onerror = null;
            this.client.onmessage = null;
            this.client = null;
            this.eventEmitter.emit('disconnected');
            if (!isForced) {
                this.tryReconnect();
            }
        }
    };
    SubscriptionClient.prototype.request = function (request) {
        var _a;
        var getObserver = this.getObserver.bind(this);
        var executeOperation = this.executeOperation.bind(this);
        var unsubscribe = this.unsubscribe.bind(this);
        var opId;
        this.clearInactivityTimeout();
        return _a = {},
            _a[symbol_observable_1.default] = function () {
                return this;
            },
            _a.subscribe = function (observerOrNext, onError, onComplete) {
                var observer = getObserver(observerOrNext, onError, onComplete);
                opId = executeOperation(request, function (error, result) {
                    if (error === null && result === null) {
                        if (observer.complete) {
                            observer.complete();
                        }
                    }
                    else if (error) {
                        if (observer.error) {
                            observer.error(error[0]);
                        }
                    }
                    else {
                        if (observer.next) {
                            observer.next(result);
                        }
                    }
                });
                return {
                    unsubscribe: function () {
                        if (opId) {
                            unsubscribe(opId);
                            opId = null;
                        }
                    },
                };
            },
            _a;
    };
    SubscriptionClient.prototype.on = function (eventName, callback, context) {
        var handler = this.eventEmitter.on(eventName, callback, context);
        return function () {
            handler.off(eventName, callback, context);
        };
    };
    SubscriptionClient.prototype.onConnected = function (callback, context) {
        return this.on('connected', callback, context);
    };
    SubscriptionClient.prototype.onConnecting = function (callback, context) {
        return this.on('connecting', callback, context);
    };
    SubscriptionClient.prototype.onDisconnected = function (callback, context) {
        return this.on('disconnected', callback, context);
    };
    SubscriptionClient.prototype.onReconnected = function (callback, context) {
        return this.on('reconnected', callback, context);
    };
    SubscriptionClient.prototype.onReconnecting = function (callback, context) {
        return this.on('reconnecting', callback, context);
    };
    SubscriptionClient.prototype.onError = function (callback, context) {
        return this.on('error', callback, context);
    };
    SubscriptionClient.prototype.unsubscribeAll = function () {
        var _this = this;
        Object.keys(this.operations).forEach(function (subId) {
            _this.unsubscribe(subId);
        });
    };
    SubscriptionClient.prototype.applyMiddlewares = function (options) {
        var _this = this;
        return new Promise(function (resolve, reject) {
            var queue = function (funcs, scope) {
                var next = function (error) {
                    if (error) {
                        reject(error);
                    }
                    else {
                        if (funcs.length > 0) {
                            var f = funcs.shift();
                            if (f) {
                                f.applyMiddleware.apply(scope, [options, next]);
                            }
                        }
                        else {
                            resolve(options);
                        }
                    }
                };
                next();
            };
            queue(__spreadArray([], _this.middlewares, true), _this);
        });
    };
    SubscriptionClient.prototype.use = function (middlewares) {
        var _this = this;
        middlewares.map(function (middleware) {
            if (typeof middleware.applyMiddleware === 'function') {
                _this.middlewares.push(middleware);
            }
            else {
                throw new Error('Middleware must implement the applyMiddleware function.');
            }
        });
        return this;
    };
    SubscriptionClient.prototype.getConnectionParams = function (connectionParams) {
        return function () { return new Promise(function (resolve, reject) {
            if (typeof connectionParams === 'function') {
                try {
                    return resolve(connectionParams.call(null));
                }
                catch (error) {
                    return reject(error);
                }
            }
            resolve(connectionParams);
        }); };
    };
    SubscriptionClient.prototype.executeOperation = function (options, handler) {
        var _this = this;
        if (this.client === null) {
            this.connect();
        }
        var opId = this.generateOperationId();
        this.operations[opId] = { options: options, handler: handler };
        this.applyMiddlewares(options)
            .then(function (processedOptions) {
            _this.checkOperationOptions(processedOptions, handler);
            if (_this.operations[opId]) {
                _this.operations[opId] = { options: processedOptions, handler: handler };
                _this.sendMessage(opId, message_types_1.default.GQL_START, processedOptions);
            }
        })
            .catch(function (error) {
            _this.unsubscribe(opId);
            handler(_this.formatErrors(error));
        });
        return opId;
    };
    SubscriptionClient.prototype.getObserver = function (observerOrNext, error, complete) {
        if (typeof observerOrNext === 'function') {
            return {
                next: function (v) { return observerOrNext(v); },
                error: function (e) { return error && error(e); },
                complete: function () { return complete && complete(); },
            };
        }
        return observerOrNext;
    };
    SubscriptionClient.prototype.createMaxConnectTimeGenerator = function () {
        var minValue = this.minWsTimeout;
        var maxValue = this.wsTimeout;
        return new Backoff({
            min: minValue,
            max: maxValue,
            factor: 1.2,
        });
    };
    SubscriptionClient.prototype.clearCheckConnectionInterval = function () {
        if (this.checkConnectionIntervalId) {
            clearInterval(this.checkConnectionIntervalId);
            this.checkConnectionIntervalId = null;
        }
    };
    SubscriptionClient.prototype.clearMaxConnectTimeout = function () {
        if (this.maxConnectTimeoutId) {
            clearTimeout(this.maxConnectTimeoutId);
            this.maxConnectTimeoutId = null;
        }
    };
    SubscriptionClient.prototype.clearTryReconnectTimeout = function () {
        if (this.tryReconnectTimeoutId) {
            clearTimeout(this.tryReconnectTimeoutId);
            this.tryReconnectTimeoutId = null;
        }
    };
    SubscriptionClient.prototype.clearInactivityTimeout = function () {
        if (this.inactivityTimeoutId) {
            clearTimeout(this.inactivityTimeoutId);
            this.inactivityTimeoutId = null;
        }
    };
    SubscriptionClient.prototype.setInactivityTimeout = function () {
        var _this = this;
        if (this.inactivityTimeout > 0 && Object.keys(this.operations).length === 0) {
            this.inactivityTimeoutId = setTimeout(function () {
                if (Object.keys(_this.operations).length === 0) {
                    _this.close();
                }
            }, this.inactivityTimeout);
        }
    };
    SubscriptionClient.prototype.checkOperationOptions = function (options, handler) {
        var query = options.query, variables = options.variables, operationName = options.operationName;
        if (!query) {
            throw new Error('Must provide a query.');
        }
        if (!handler) {
            throw new Error('Must provide an handler.');
        }
        if ((!(0, is_string_1.default)(query) && !(0, getOperationAST_1.getOperationAST)(query, operationName)) ||
            (operationName && !(0, is_string_1.default)(operationName)) ||
            (variables && !(0, is_object_1.default)(variables))) {
            throw new Error('Incorrect option types. query must be a string or a document,' +
                '`operationName` must be a string, and `variables` must be an object.');
        }
    };
    SubscriptionClient.prototype.buildMessage = function (id, type, payload) {
        var payloadToReturn = payload && payload.query ? __assign$1(__assign$1({}, payload), { query: typeof payload.query === 'string' ? payload.query : (0, printer_1.print)(payload.query) }) :
            payload;
        return {
            id: id,
            type: type,
            payload: payloadToReturn,
        };
    };
    SubscriptionClient.prototype.formatErrors = function (errors) {
        if (Array.isArray(errors)) {
            return errors;
        }
        if (errors && errors.errors) {
            return this.formatErrors(errors.errors);
        }
        if (errors && errors.message) {
            return [errors];
        }
        return [{
                name: 'FormatedError',
                message: 'Unknown error',
                originalError: errors,
            }];
    };
    SubscriptionClient.prototype.sendMessage = function (id, type, payload) {
        this.sendMessageRaw(this.buildMessage(id, type, payload));
    };
    SubscriptionClient.prototype.sendMessageRaw = function (message) {
        switch (this.status) {
            case this.wsImpl.OPEN:
                var serializedMessage = JSON.stringify(message);
                try {
                    JSON.parse(serializedMessage);
                }
                catch (e) {
                    this.eventEmitter.emit('error', new Error("Message must be JSON-serializable. Got: " + message));
                }
                this.client.send(serializedMessage);
                break;
            case this.wsImpl.CONNECTING:
                this.unsentMessagesQueue.push(message);
                break;
            default:
                if (!this.reconnecting) {
                    this.eventEmitter.emit('error', new Error('A message was not sent because socket is not connected, is closing or ' +
                        'is already closed. Message was: ' + JSON.stringify(message)));
                }
        }
    };
    SubscriptionClient.prototype.generateOperationId = function () {
        return String(++this.nextOperationId);
    };
    SubscriptionClient.prototype.tryReconnect = function () {
        var _this = this;
        if (!this.reconnect || this.backoff.attempts >= this.reconnectionAttempts) {
            return;
        }
        if (!this.reconnecting) {
            Object.keys(this.operations).forEach(function (key) {
                _this.unsentMessagesQueue.push(_this.buildMessage(key, message_types_1.default.GQL_START, _this.operations[key].options));
            });
            this.reconnecting = true;
        }
        this.clearTryReconnectTimeout();
        var delay = this.backoff.duration();
        this.tryReconnectTimeoutId = setTimeout(function () {
            _this.connect();
        }, delay);
    };
    SubscriptionClient.prototype.flushUnsentMessagesQueue = function () {
        var _this = this;
        this.unsentMessagesQueue.forEach(function (message) {
            _this.sendMessageRaw(message);
        });
        this.unsentMessagesQueue = [];
    };
    SubscriptionClient.prototype.checkConnection = function () {
        if (this.wasKeepAliveReceived) {
            this.wasKeepAliveReceived = false;
            return;
        }
        if (!this.reconnecting) {
            this.close(false, true);
        }
    };
    SubscriptionClient.prototype.checkMaxConnectTimeout = function () {
        var _this = this;
        this.clearMaxConnectTimeout();
        this.maxConnectTimeoutId = setTimeout(function () {
            if (_this.status !== _this.wsImpl.OPEN) {
                _this.reconnecting = true;
                _this.close(false, true);
            }
        }, this.maxConnectTimeGenerator.duration());
    };
    SubscriptionClient.prototype.connect = function () {
        var _a;
        var _this = this;
        this.client = new ((_a = this.wsImpl).bind.apply(_a, __spreadArray([void 0, this.url, this.wsProtocols], this.wsOptionArguments, false)))();
        this.checkMaxConnectTimeout();
        this.client.onopen = function () { return __awaiter(_this, void 0, void 0, function () {
            var connectionParams, error_1;
            return __generator(this, function (_a) {
                switch (_a.label) {
                    case 0:
                        if (!(this.status === this.wsImpl.OPEN)) return [3, 4];
                        this.clearMaxConnectTimeout();
                        this.closedByUser = false;
                        this.eventEmitter.emit(this.reconnecting ? 'reconnecting' : 'connecting');
                        _a.label = 1;
                    case 1:
                        _a.trys.push([1, 3, , 4]);
                        return [4, this.connectionParams()];
                    case 2:
                        connectionParams = _a.sent();
                        this.sendMessage(undefined, message_types_1.default.GQL_CONNECTION_INIT, connectionParams);
                        this.flushUnsentMessagesQueue();
                        return [3, 4];
                    case 3:
                        error_1 = _a.sent();
                        this.sendMessage(undefined, message_types_1.default.GQL_CONNECTION_ERROR, error_1);
                        this.flushUnsentMessagesQueue();
                        return [3, 4];
                    case 4: return [2];
                }
            });
        }); };
        this.client.onclose = function () {
            if (!_this.closedByUser) {
                _this.close(false, false);
            }
        };
        this.client.onerror = function (err) {
            _this.eventEmitter.emit('error', err);
        };
        this.client.onmessage = function (_a) {
            var data = _a.data;
            _this.processReceivedData(data);
        };
    };
    SubscriptionClient.prototype.processReceivedData = function (receivedData) {
        var parsedMessage;
        var opId;
        try {
            parsedMessage = JSON.parse(receivedData);
            opId = parsedMessage.id;
        }
        catch (e) {
            throw new Error("Message must be JSON-parseable. Got: " + receivedData);
        }
        if ([message_types_1.default.GQL_DATA,
            message_types_1.default.GQL_COMPLETE,
            message_types_1.default.GQL_ERROR,
        ].indexOf(parsedMessage.type) !== -1 && !this.operations[opId]) {
            this.unsubscribe(opId);
            return;
        }
        switch (parsedMessage.type) {
            case message_types_1.default.GQL_CONNECTION_ERROR:
                if (this.connectionCallback) {
                    this.connectionCallback(parsedMessage.payload);
                }
                break;
            case message_types_1.default.GQL_CONNECTION_ACK:
                this.eventEmitter.emit(this.reconnecting ? 'reconnected' : 'connected', parsedMessage.payload);
                this.reconnecting = false;
                this.backoff.reset();
                this.maxConnectTimeGenerator.reset();
                if (this.connectionCallback) {
                    this.connectionCallback();
                }
                break;
            case message_types_1.default.GQL_COMPLETE:
                var handler = this.operations[opId].handler;
                delete this.operations[opId];
                handler.call(this, null, null);
                break;
            case message_types_1.default.GQL_ERROR:
                this.operations[opId].handler(this.formatErrors(parsedMessage.payload), null);
                delete this.operations[opId];
                break;
            case message_types_1.default.GQL_DATA:
                var parsedPayload = !parsedMessage.payload.errors ?
                    parsedMessage.payload : __assign$1(__assign$1({}, parsedMessage.payload), { errors: this.formatErrors(parsedMessage.payload.errors) });
                this.operations[opId].handler(null, parsedPayload);
                break;
            case message_types_1.default.GQL_CONNECTION_KEEP_ALIVE:
                var firstKA = typeof this.wasKeepAliveReceived === 'undefined';
                this.wasKeepAliveReceived = true;
                if (firstKA) {
                    this.checkConnection();
                }
                if (this.checkConnectionIntervalId) {
                    clearInterval(this.checkConnectionIntervalId);
                    this.checkConnection();
                }
                this.checkConnectionIntervalId = setInterval(this.checkConnection.bind(this), this.wsTimeout);
                break;
            default:
                throw new Error('Invalid message type!');
        }
    };
    SubscriptionClient.prototype.unsubscribe = function (opId) {
        if (this.operations[opId]) {
            delete this.operations[opId];
            this.setInactivityTimeout();
            this.sendMessage(opId, message_types_1.default.GQL_STOP, undefined);
        }
    };
    return SubscriptionClient;
}());
SubscriptionClient_1 = client.SubscriptionClient = SubscriptionClient;

var WebSocketLink = (function (_super) {
    __extends(WebSocketLink, _super);
    function WebSocketLink(paramsOrClient) {
        var _this = _super.call(this) || this;
        if (paramsOrClient instanceof SubscriptionClient_1) {
            _this.subscriptionClient = paramsOrClient;
        }
        else {
            _this.subscriptionClient = new SubscriptionClient_1(paramsOrClient.uri, paramsOrClient.options, paramsOrClient.webSocketImpl);
        }
        return _this;
    }
    WebSocketLink.prototype.request = function (operation) {
        return this.subscriptionClient.request(operation);
    };
    return WebSocketLink;
}(ApolloLink));

/******************************************************************************
Copyright (c) Microsoft Corporation.

Permission to use, copy, modify, and/or distribute this software for any
purpose with or without fee is hereby granted.

THE SOFTWARE IS PROVIDED "AS IS" AND THE AUTHOR DISCLAIMS ALL WARRANTIES WITH
REGARD TO THIS SOFTWARE INCLUDING ALL IMPLIED WARRANTIES OF MERCHANTABILITY
AND FITNESS. IN NO EVENT SHALL THE AUTHOR BE LIABLE FOR ANY SPECIAL, DIRECT,
INDIRECT, OR CONSEQUENTIAL DAMAGES OR ANY DAMAGES WHATSOEVER RESULTING FROM
LOSS OF USE, DATA OR PROFITS, WHETHER IN AN ACTION OF CONTRACT, NEGLIGENCE OR
OTHER TORTIOUS ACTION, ARISING OUT OF OR IN CONNECTION WITH THE USE OR
PERFORMANCE OF THIS SOFTWARE.
***************************************************************************** */

var __assign = function() {
  __assign = Object.assign || function __assign(t) {
      for (var s, i = 1, n = arguments.length; i < n; i++) {
          s = arguments[i];
          for (var p in s) if (Object.prototype.hasOwnProperty.call(s, p)) t[p] = s[p];
      }
      return t;
  };
  return __assign.apply(this, arguments);
};

typeof SuppressedError === "function" ? SuppressedError : function (error, suppressed, message) {
  var e = new Error(message);
  return e.name = "SuppressedError", e.error = error, e.suppressed = suppressed, e;
};

function _typeof$2(obj) { "@babel/helpers - typeof"; if (typeof Symbol === "function" && typeof Symbol.iterator === "symbol") { _typeof$2 = function _typeof(obj) { return typeof obj; }; } else { _typeof$2 = function _typeof(obj) { return obj && typeof Symbol === "function" && obj.constructor === Symbol && obj !== Symbol.prototype ? "symbol" : typeof obj; }; } return _typeof$2(obj); }

/**
 * Return true if `value` is object-like. A value is object-like if it's not
 * `null` and has a `typeof` result of "object".
 */
function isObjectLike(value) {
  return _typeof$2(value) == 'object' && value !== null;
}

// In ES2015 (or a polyfilled) environment, this will be Symbol.iterator

var SYMBOL_TO_STRING_TAG = typeof Symbol === 'function' && Symbol.toStringTag != null ? Symbol.toStringTag : '@@toStringTag';

/**
 * Represents a location in a Source.
 */

/**
 * Takes a Source and a UTF-8 character offset, and returns the corresponding
 * line and column as a SourceLocation.
 */
function getLocation(source, position) {
  var lineRegexp = /\r\n|[\n\r]/g;
  var line = 1;
  var column = position + 1;
  var match;

  while ((match = lineRegexp.exec(source.body)) && match.index < position) {
    line += 1;
    column = position + 1 - (match.index + match[0].length);
  }

  return {
    line: line,
    column: column
  };
}

/**
 * Render a helpful description of the location in the GraphQL Source document.
 */

function printLocation(location) {
  return printSourceLocation(location.source, getLocation(location.source, location.start));
}
/**
 * Render a helpful description of the location in the GraphQL Source document.
 */

function printSourceLocation(source, sourceLocation) {
  var firstLineColumnOffset = source.locationOffset.column - 1;
  var body = whitespace(firstLineColumnOffset) + source.body;
  var lineIndex = sourceLocation.line - 1;
  var lineOffset = source.locationOffset.line - 1;
  var lineNum = sourceLocation.line + lineOffset;
  var columnOffset = sourceLocation.line === 1 ? firstLineColumnOffset : 0;
  var columnNum = sourceLocation.column + columnOffset;
  var locationStr = "".concat(source.name, ":").concat(lineNum, ":").concat(columnNum, "\n");
  var lines = body.split(/\r\n|[\n\r]/g);
  var locationLine = lines[lineIndex]; // Special case for minified documents

  if (locationLine.length > 120) {
    var subLineIndex = Math.floor(columnNum / 80);
    var subLineColumnNum = columnNum % 80;
    var subLines = [];

    for (var i = 0; i < locationLine.length; i += 80) {
      subLines.push(locationLine.slice(i, i + 80));
    }

    return locationStr + printPrefixedLines([["".concat(lineNum), subLines[0]]].concat(subLines.slice(1, subLineIndex + 1).map(function (subLine) {
      return ['', subLine];
    }), [[' ', whitespace(subLineColumnNum - 1) + '^'], ['', subLines[subLineIndex + 1]]]));
  }

  return locationStr + printPrefixedLines([// Lines specified like this: ["prefix", "string"],
  ["".concat(lineNum - 1), lines[lineIndex - 1]], ["".concat(lineNum), locationLine], ['', whitespace(columnNum - 1) + '^'], ["".concat(lineNum + 1), lines[lineIndex + 1]]]);
}

function printPrefixedLines(lines) {
  var existingLines = lines.filter(function (_ref) {
    _ref[0];
        var line = _ref[1];
    return line !== undefined;
  });
  var padLen = Math.max.apply(Math, existingLines.map(function (_ref2) {
    var prefix = _ref2[0];
    return prefix.length;
  }));
  return existingLines.map(function (_ref3) {
    var prefix = _ref3[0],
        line = _ref3[1];
    return leftPad(padLen, prefix) + (line ? ' | ' + line : ' |');
  }).join('\n');
}

function whitespace(len) {
  return Array(len + 1).join(' ');
}

function leftPad(len, str) {
  return whitespace(len - str.length) + str;
}

function _typeof$1(obj) { "@babel/helpers - typeof"; if (typeof Symbol === "function" && typeof Symbol.iterator === "symbol") { _typeof$1 = function _typeof(obj) { return typeof obj; }; } else { _typeof$1 = function _typeof(obj) { return obj && typeof Symbol === "function" && obj.constructor === Symbol && obj !== Symbol.prototype ? "symbol" : typeof obj; }; } return _typeof$1(obj); }

function ownKeys(object, enumerableOnly) { var keys = Object.keys(object); if (Object.getOwnPropertySymbols) { var symbols = Object.getOwnPropertySymbols(object); if (enumerableOnly) symbols = symbols.filter(function (sym) { return Object.getOwnPropertyDescriptor(object, sym).enumerable; }); keys.push.apply(keys, symbols); } return keys; }

function _objectSpread(target) { for (var i = 1; i < arguments.length; i++) { var source = arguments[i] != null ? arguments[i] : {}; if (i % 2) { ownKeys(Object(source), true).forEach(function (key) { _defineProperty(target, key, source[key]); }); } else if (Object.getOwnPropertyDescriptors) { Object.defineProperties(target, Object.getOwnPropertyDescriptors(source)); } else { ownKeys(Object(source)).forEach(function (key) { Object.defineProperty(target, key, Object.getOwnPropertyDescriptor(source, key)); }); } } return target; }

function _defineProperty(obj, key, value) { if (key in obj) { Object.defineProperty(obj, key, { value: value, enumerable: true, configurable: true, writable: true }); } else { obj[key] = value; } return obj; }

function _classCallCheck(instance, Constructor) { if (!(instance instanceof Constructor)) { throw new TypeError("Cannot call a class as a function"); } }

function _defineProperties$1(target, props) { for (var i = 0; i < props.length; i++) { var descriptor = props[i]; descriptor.enumerable = descriptor.enumerable || false; descriptor.configurable = true; if ("value" in descriptor) descriptor.writable = true; Object.defineProperty(target, descriptor.key, descriptor); } }

function _createClass$1(Constructor, protoProps, staticProps) { if (protoProps) _defineProperties$1(Constructor.prototype, protoProps); if (staticProps) _defineProperties$1(Constructor, staticProps); return Constructor; }

function _inherits(subClass, superClass) { if (typeof superClass !== "function" && superClass !== null) { throw new TypeError("Super expression must either be null or a function"); } subClass.prototype = Object.create(superClass && superClass.prototype, { constructor: { value: subClass, writable: true, configurable: true } }); if (superClass) _setPrototypeOf(subClass, superClass); }

function _createSuper(Derived) { var hasNativeReflectConstruct = _isNativeReflectConstruct(); return function _createSuperInternal() { var Super = _getPrototypeOf(Derived), result; if (hasNativeReflectConstruct) { var NewTarget = _getPrototypeOf(this).constructor; result = Reflect.construct(Super, arguments, NewTarget); } else { result = Super.apply(this, arguments); } return _possibleConstructorReturn(this, result); }; }

function _possibleConstructorReturn(self, call) { if (call && (_typeof$1(call) === "object" || typeof call === "function")) { return call; } return _assertThisInitialized(self); }

function _assertThisInitialized(self) { if (self === void 0) { throw new ReferenceError("this hasn't been initialised - super() hasn't been called"); } return self; }

function _wrapNativeSuper(Class) { var _cache = typeof Map === "function" ? new Map() : undefined; _wrapNativeSuper = function _wrapNativeSuper(Class) { if (Class === null || !_isNativeFunction(Class)) return Class; if (typeof Class !== "function") { throw new TypeError("Super expression must either be null or a function"); } if (typeof _cache !== "undefined") { if (_cache.has(Class)) return _cache.get(Class); _cache.set(Class, Wrapper); } function Wrapper() { return _construct(Class, arguments, _getPrototypeOf(this).constructor); } Wrapper.prototype = Object.create(Class.prototype, { constructor: { value: Wrapper, enumerable: false, writable: true, configurable: true } }); return _setPrototypeOf(Wrapper, Class); }; return _wrapNativeSuper(Class); }

function _construct(Parent, args, Class) { if (_isNativeReflectConstruct()) { _construct = Reflect.construct; } else { _construct = function _construct(Parent, args, Class) { var a = [null]; a.push.apply(a, args); var Constructor = Function.bind.apply(Parent, a); var instance = new Constructor(); if (Class) _setPrototypeOf(instance, Class.prototype); return instance; }; } return _construct.apply(null, arguments); }

function _isNativeReflectConstruct() { if (typeof Reflect === "undefined" || !Reflect.construct) return false; if (Reflect.construct.sham) return false; if (typeof Proxy === "function") return true; try { Date.prototype.toString.call(Reflect.construct(Date, [], function () {})); return true; } catch (e) { return false; } }

function _isNativeFunction(fn) { return Function.toString.call(fn).indexOf("[native code]") !== -1; }

function _setPrototypeOf(o, p) { _setPrototypeOf = Object.setPrototypeOf || function _setPrototypeOf(o, p) { o.__proto__ = p; return o; }; return _setPrototypeOf(o, p); }

function _getPrototypeOf(o) { _getPrototypeOf = Object.setPrototypeOf ? Object.getPrototypeOf : function _getPrototypeOf(o) { return o.__proto__ || Object.getPrototypeOf(o); }; return _getPrototypeOf(o); }
/**
 * A GraphQLError describes an Error found during the parse, validate, or
 * execute phases of performing a GraphQL operation. In addition to a message
 * and stack trace, it also includes information about the locations in a
 * GraphQL document and/or execution result that correspond to the Error.
 */

var GraphQLError = /*#__PURE__*/function (_Error) {
  _inherits(GraphQLError, _Error);

  var _super = _createSuper(GraphQLError);

  /**
   * An array of { line, column } locations within the source GraphQL document
   * which correspond to this error.
   *
   * Errors during validation often contain multiple locations, for example to
   * point out two things with the same name. Errors during execution include a
   * single location, the field which produced the error.
   *
   * Enumerable, and appears in the result of JSON.stringify().
   */

  /**
   * An array describing the JSON-path into the execution response which
   * corresponds to this error. Only included for errors during execution.
   *
   * Enumerable, and appears in the result of JSON.stringify().
   */

  /**
   * An array of GraphQL AST Nodes corresponding to this error.
   */

  /**
   * The source GraphQL document for the first location of this error.
   *
   * Note that if this Error represents more than one node, the source may not
   * represent nodes after the first node.
   */

  /**
   * An array of character offsets within the source GraphQL document
   * which correspond to this error.
   */

  /**
   * The original error thrown from a field resolver during execution.
   */

  /**
   * Extension fields to add to the formatted error.
   */
  function GraphQLError(message, nodes, source, positions, path, originalError, extensions) {
    var _nodeLocations, _nodeLocations2, _nodeLocations3;

    var _this;

    _classCallCheck(this, GraphQLError);

    _this = _super.call(this, message);
    _this.name = 'GraphQLError';
    _this.originalError = originalError !== null && originalError !== void 0 ? originalError : undefined; // Compute list of blame nodes.

    _this.nodes = undefinedIfEmpty(Array.isArray(nodes) ? nodes : nodes ? [nodes] : undefined);
    var nodeLocations = [];

    for (var _i2 = 0, _ref3 = (_this$nodes = _this.nodes) !== null && _this$nodes !== void 0 ? _this$nodes : []; _i2 < _ref3.length; _i2++) {
      var _this$nodes;

      var _ref4 = _ref3[_i2];
      var loc = _ref4.loc;

      if (loc != null) {
        nodeLocations.push(loc);
      }
    }

    nodeLocations = undefinedIfEmpty(nodeLocations); // Compute locations in the source for the given nodes/positions.

    _this.source = source !== null && source !== void 0 ? source : (_nodeLocations = nodeLocations) === null || _nodeLocations === void 0 ? void 0 : _nodeLocations[0].source;
    _this.positions = positions !== null && positions !== void 0 ? positions : (_nodeLocations2 = nodeLocations) === null || _nodeLocations2 === void 0 ? void 0 : _nodeLocations2.map(function (loc) {
      return loc.start;
    });
    _this.locations = positions && source ? positions.map(function (pos) {
      return getLocation(source, pos);
    }) : (_nodeLocations3 = nodeLocations) === null || _nodeLocations3 === void 0 ? void 0 : _nodeLocations3.map(function (loc) {
      return getLocation(loc.source, loc.start);
    });
    _this.path = path !== null && path !== void 0 ? path : undefined;
    var originalExtensions = originalError === null || originalError === void 0 ? void 0 : originalError.extensions;

    if (extensions == null && isObjectLike(originalExtensions)) {
      _this.extensions = _objectSpread({}, originalExtensions);
    } else {
      _this.extensions = extensions !== null && extensions !== void 0 ? extensions : {};
    } // By being enumerable, JSON.stringify will include bellow properties in the resulting output.
    // This ensures that the simplest possible GraphQL service adheres to the spec.


    Object.defineProperties(_assertThisInitialized(_this), {
      message: {
        enumerable: true
      },
      locations: {
        enumerable: _this.locations != null
      },
      path: {
        enumerable: _this.path != null
      },
      extensions: {
        enumerable: _this.extensions != null && Object.keys(_this.extensions).length > 0
      },
      name: {
        enumerable: false
      },
      nodes: {
        enumerable: false
      },
      source: {
        enumerable: false
      },
      positions: {
        enumerable: false
      },
      originalError: {
        enumerable: false
      }
    }); // Include (non-enumerable) stack trace.

    if (originalError !== null && originalError !== void 0 && originalError.stack) {
      Object.defineProperty(_assertThisInitialized(_this), 'stack', {
        value: originalError.stack,
        writable: true,
        configurable: true
      });
      return _possibleConstructorReturn(_this);
    } // istanbul ignore next (See: 'https://github.com/graphql/graphql-js/issues/2317')


    if (Error.captureStackTrace) {
      Error.captureStackTrace(_assertThisInitialized(_this), GraphQLError);
    } else {
      Object.defineProperty(_assertThisInitialized(_this), 'stack', {
        value: Error().stack,
        writable: true,
        configurable: true
      });
    }

    return _this;
  }

  _createClass$1(GraphQLError, [{
    key: "toString",
    value: function toString() {
      return printError(this);
    } // FIXME: workaround to not break chai comparisons, should be remove in v16
    // $FlowFixMe[unsupported-syntax] Flow doesn't support computed properties yet

  }, {
    key: SYMBOL_TO_STRING_TAG,
    get: function get() {
      return 'Object';
    }
  }]);

  return GraphQLError;
}( /*#__PURE__*/_wrapNativeSuper(Error));

function undefinedIfEmpty(array) {
  return array === undefined || array.length === 0 ? undefined : array;
}
/**
 * Prints a GraphQLError to a string, representing useful location information
 * about the error's position in the source.
 */


function printError(error) {
  var output = error.message;

  if (error.nodes) {
    for (var _i4 = 0, _error$nodes2 = error.nodes; _i4 < _error$nodes2.length; _i4++) {
      var node = _error$nodes2[_i4];

      if (node.loc) {
        output += '\n\n' + printLocation(node.loc);
      }
    }
  } else if (error.source && error.locations) {
    for (var _i6 = 0, _error$locations2 = error.locations; _i6 < _error$locations2.length; _i6++) {
      var location = _error$locations2[_i6];
      output += '\n\n' + printSourceLocation(error.source, location);
    }
  }

  return output;
}

/**
 * Produces a GraphQLError representing a syntax error, containing useful
 * descriptive information about the syntax error's position in the source.
 */

function syntaxError(source, position, description) {
  return new GraphQLError("Syntax Error: ".concat(description), undefined, source, [position]);
}

/**
 * The set of allowed kind values for AST nodes.
 */
var Kind = Object.freeze({
  // Name
  NAME: 'Name',
  // Document
  DOCUMENT: 'Document',
  OPERATION_DEFINITION: 'OperationDefinition',
  VARIABLE_DEFINITION: 'VariableDefinition',
  SELECTION_SET: 'SelectionSet',
  FIELD: 'Field',
  ARGUMENT: 'Argument',
  // Fragments
  FRAGMENT_SPREAD: 'FragmentSpread',
  INLINE_FRAGMENT: 'InlineFragment',
  FRAGMENT_DEFINITION: 'FragmentDefinition',
  // Values
  VARIABLE: 'Variable',
  INT: 'IntValue',
  FLOAT: 'FloatValue',
  STRING: 'StringValue',
  BOOLEAN: 'BooleanValue',
  NULL: 'NullValue',
  ENUM: 'EnumValue',
  LIST: 'ListValue',
  OBJECT: 'ObjectValue',
  OBJECT_FIELD: 'ObjectField',
  // Directives
  DIRECTIVE: 'Directive',
  // Types
  NAMED_TYPE: 'NamedType',
  LIST_TYPE: 'ListType',
  NON_NULL_TYPE: 'NonNullType',
  // Type System Definitions
  SCHEMA_DEFINITION: 'SchemaDefinition',
  OPERATION_TYPE_DEFINITION: 'OperationTypeDefinition',
  // Type Definitions
  SCALAR_TYPE_DEFINITION: 'ScalarTypeDefinition',
  OBJECT_TYPE_DEFINITION: 'ObjectTypeDefinition',
  FIELD_DEFINITION: 'FieldDefinition',
  INPUT_VALUE_DEFINITION: 'InputValueDefinition',
  INTERFACE_TYPE_DEFINITION: 'InterfaceTypeDefinition',
  UNION_TYPE_DEFINITION: 'UnionTypeDefinition',
  ENUM_TYPE_DEFINITION: 'EnumTypeDefinition',
  ENUM_VALUE_DEFINITION: 'EnumValueDefinition',
  INPUT_OBJECT_TYPE_DEFINITION: 'InputObjectTypeDefinition',
  // Directive Definitions
  DIRECTIVE_DEFINITION: 'DirectiveDefinition',
  // Type System Extensions
  SCHEMA_EXTENSION: 'SchemaExtension',
  // Type Extensions
  SCALAR_TYPE_EXTENSION: 'ScalarTypeExtension',
  OBJECT_TYPE_EXTENSION: 'ObjectTypeExtension',
  INTERFACE_TYPE_EXTENSION: 'InterfaceTypeExtension',
  UNION_TYPE_EXTENSION: 'UnionTypeExtension',
  ENUM_TYPE_EXTENSION: 'EnumTypeExtension',
  INPUT_OBJECT_TYPE_EXTENSION: 'InputObjectTypeExtension'
});
/**
 * The enum type representing the possible kind values of AST nodes.
 */

function invariant(condition, message) {
  var booleanCondition = Boolean(condition); // istanbul ignore else (See transformation done in './resources/inlineInvariant.js')

  if (!booleanCondition) {
    throw new Error(message != null ? message : 'Unexpected invariant triggered.');
  }
}

// istanbul ignore next (See: 'https://github.com/graphql/graphql-js/issues/2317')
var nodejsCustomInspectSymbol = typeof Symbol === 'function' && typeof Symbol.for === 'function' ? Symbol.for('nodejs.util.inspect.custom') : undefined;
var nodejsCustomInspectSymbol$1 = nodejsCustomInspectSymbol;

/**
 * The `defineInspect()` function defines `inspect()` prototype method as alias of `toJSON`
 */

function defineInspect(classObject) {
  var fn = classObject.prototype.toJSON;
  typeof fn === 'function' || invariant(0);
  classObject.prototype.inspect = fn; // istanbul ignore else (See: 'https://github.com/graphql/graphql-js/issues/2317')

  if (nodejsCustomInspectSymbol$1) {
    classObject.prototype[nodejsCustomInspectSymbol$1] = fn;
  }
}

/**
 * Contains a range of UTF-8 character offsets and token references that
 * identify the region of the source from which the AST derived.
 */
var Location = /*#__PURE__*/function () {
  /**
   * The character offset at which this Node begins.
   */

  /**
   * The character offset at which this Node ends.
   */

  /**
   * The Token at which this Node begins.
   */

  /**
   * The Token at which this Node ends.
   */

  /**
   * The Source document the AST represents.
   */
  function Location(startToken, endToken, source) {
    this.start = startToken.start;
    this.end = endToken.end;
    this.startToken = startToken;
    this.endToken = endToken;
    this.source = source;
  }

  var _proto = Location.prototype;

  _proto.toJSON = function toJSON() {
    return {
      start: this.start,
      end: this.end
    };
  };

  return Location;
}(); // Print a simplified form when appearing in `inspect` and `util.inspect`.

defineInspect(Location);
/**
 * Represents a range of characters represented by a lexical token
 * within a Source.
 */

var Token = /*#__PURE__*/function () {
  /**
   * The kind of Token.
   */

  /**
   * The character offset at which this Node begins.
   */

  /**
   * The character offset at which this Node ends.
   */

  /**
   * The 1-indexed line number on which this Token appears.
   */

  /**
   * The 1-indexed column number at which this Token begins.
   */

  /**
   * For non-punctuation tokens, represents the interpreted value of the token.
   */

  /**
   * Tokens exist as nodes in a double-linked-list amongst all tokens
   * including ignored tokens. <SOF> is always the first node and <EOF>
   * the last.
   */
  function Token(kind, start, end, line, column, prev, value) {
    this.kind = kind;
    this.start = start;
    this.end = end;
    this.line = line;
    this.column = column;
    this.value = value;
    this.prev = prev;
    this.next = null;
  }

  var _proto2 = Token.prototype;

  _proto2.toJSON = function toJSON() {
    return {
      kind: this.kind,
      value: this.value,
      line: this.line,
      column: this.column
    };
  };

  return Token;
}(); // Print a simplified form when appearing in `inspect` and `util.inspect`.

defineInspect(Token);
/**
 * The list of all possible AST node types.
 */

/**
 * An exported enum describing the different kinds of tokens that the
 * lexer emits.
 */
var TokenKind = Object.freeze({
  SOF: '<SOF>',
  EOF: '<EOF>',
  BANG: '!',
  DOLLAR: '$',
  AMP: '&',
  PAREN_L: '(',
  PAREN_R: ')',
  SPREAD: '...',
  COLON: ':',
  EQUALS: '=',
  AT: '@',
  BRACKET_L: '[',
  BRACKET_R: ']',
  BRACE_L: '{',
  PIPE: '|',
  BRACE_R: '}',
  NAME: 'Name',
  INT: 'Int',
  FLOAT: 'Float',
  STRING: 'String',
  BLOCK_STRING: 'BlockString',
  COMMENT: 'Comment'
});
/**
 * The enum type representing the token kinds values.
 */

function _typeof(obj) { "@babel/helpers - typeof"; if (typeof Symbol === "function" && typeof Symbol.iterator === "symbol") { _typeof = function _typeof(obj) { return typeof obj; }; } else { _typeof = function _typeof(obj) { return obj && typeof Symbol === "function" && obj.constructor === Symbol && obj !== Symbol.prototype ? "symbol" : typeof obj; }; } return _typeof(obj); }
var MAX_ARRAY_LENGTH = 10;
var MAX_RECURSIVE_DEPTH = 2;
/**
 * Used to print values in error messages.
 */

function inspect(value) {
  return formatValue(value, []);
}

function formatValue(value, seenValues) {
  switch (_typeof(value)) {
    case 'string':
      return JSON.stringify(value);

    case 'function':
      return value.name ? "[function ".concat(value.name, "]") : '[function]';

    case 'object':
      if (value === null) {
        return 'null';
      }

      return formatObjectValue(value, seenValues);

    default:
      return String(value);
  }
}

function formatObjectValue(value, previouslySeenValues) {
  if (previouslySeenValues.indexOf(value) !== -1) {
    return '[Circular]';
  }

  var seenValues = [].concat(previouslySeenValues, [value]);
  var customInspectFn = getCustomFn(value);

  if (customInspectFn !== undefined) {
    var customValue = customInspectFn.call(value); // check for infinite recursion

    if (customValue !== value) {
      return typeof customValue === 'string' ? customValue : formatValue(customValue, seenValues);
    }
  } else if (Array.isArray(value)) {
    return formatArray(value, seenValues);
  }

  return formatObject(value, seenValues);
}

function formatObject(object, seenValues) {
  var keys = Object.keys(object);

  if (keys.length === 0) {
    return '{}';
  }

  if (seenValues.length > MAX_RECURSIVE_DEPTH) {
    return '[' + getObjectTag(object) + ']';
  }

  var properties = keys.map(function (key) {
    var value = formatValue(object[key], seenValues);
    return key + ': ' + value;
  });
  return '{ ' + properties.join(', ') + ' }';
}

function formatArray(array, seenValues) {
  if (array.length === 0) {
    return '[]';
  }

  if (seenValues.length > MAX_RECURSIVE_DEPTH) {
    return '[Array]';
  }

  var len = Math.min(MAX_ARRAY_LENGTH, array.length);
  var remaining = array.length - len;
  var items = [];

  for (var i = 0; i < len; ++i) {
    items.push(formatValue(array[i], seenValues));
  }

  if (remaining === 1) {
    items.push('... 1 more item');
  } else if (remaining > 1) {
    items.push("... ".concat(remaining, " more items"));
  }

  return '[' + items.join(', ') + ']';
}

function getCustomFn(object) {
  var customInspectFn = object[String(nodejsCustomInspectSymbol$1)];

  if (typeof customInspectFn === 'function') {
    return customInspectFn;
  }

  if (typeof object.inspect === 'function') {
    return object.inspect;
  }
}

function getObjectTag(object) {
  var tag = Object.prototype.toString.call(object).replace(/^\[object /, '').replace(/]$/, '');

  if (tag === 'Object' && typeof object.constructor === 'function') {
    var name = object.constructor.name;

    if (typeof name === 'string' && name !== '') {
      return name;
    }
  }

  return tag;
}

function devAssert(condition, message) {
  var booleanCondition = Boolean(condition); // istanbul ignore else (See transformation done in './resources/inlineInvariant.js')

  if (!booleanCondition) {
    throw new Error(message);
  }
}

/**
 * A replacement for instanceof which includes an error warning when multi-realm
 * constructors are detected.
 */

// See: https://expressjs.com/en/advanced/best-practice-performance.html#set-node_env-to-production
// See: https://webpack.js.org/guides/production/
var instanceOf = // istanbul ignore next (See: 'https://github.com/graphql/graphql-js/issues/2317')
// eslint-disable-next-line no-shadow
function instanceOf(value, constructor) {
  return value instanceof constructor;
} ;

function _defineProperties(target, props) { for (var i = 0; i < props.length; i++) { var descriptor = props[i]; descriptor.enumerable = descriptor.enumerable || false; descriptor.configurable = true; if ("value" in descriptor) descriptor.writable = true; Object.defineProperty(target, descriptor.key, descriptor); } }

function _createClass(Constructor, protoProps, staticProps) { if (protoProps) _defineProperties(Constructor.prototype, protoProps); if (staticProps) _defineProperties(Constructor, staticProps); return Constructor; }

/**
 * A representation of source input to GraphQL. The `name` and `locationOffset` parameters are
 * optional, but they are useful for clients who store GraphQL documents in source files.
 * For example, if the GraphQL input starts at line 40 in a file named `Foo.graphql`, it might
 * be useful for `name` to be `"Foo.graphql"` and location to be `{ line: 40, column: 1 }`.
 * The `line` and `column` properties in `locationOffset` are 1-indexed.
 */
var Source = /*#__PURE__*/function () {
  function Source(body) {
    var name = arguments.length > 1 && arguments[1] !== undefined ? arguments[1] : 'GraphQL request';
    var locationOffset = arguments.length > 2 && arguments[2] !== undefined ? arguments[2] : {
      line: 1,
      column: 1
    };
    typeof body === 'string' || devAssert(0, "Body must be a string. Received: ".concat(inspect(body), "."));
    this.body = body;
    this.name = name;
    this.locationOffset = locationOffset;
    this.locationOffset.line > 0 || devAssert(0, 'line in locationOffset is 1-indexed and must be positive.');
    this.locationOffset.column > 0 || devAssert(0, 'column in locationOffset is 1-indexed and must be positive.');
  } // $FlowFixMe[unsupported-syntax] Flow doesn't support computed properties yet


  _createClass(Source, [{
    key: SYMBOL_TO_STRING_TAG,
    get: function get() {
      return 'Source';
    }
  }]);

  return Source;
}();
/**
 * Test if the given value is a Source object.
 *
 * @internal
 */

// eslint-disable-next-line no-redeclare
function isSource(source) {
  return instanceOf(source, Source);
}

/**
 * The set of allowed directive location values.
 */
var DirectiveLocation = Object.freeze({
  // Request Definitions
  QUERY: 'QUERY',
  MUTATION: 'MUTATION',
  SUBSCRIPTION: 'SUBSCRIPTION',
  FIELD: 'FIELD',
  FRAGMENT_DEFINITION: 'FRAGMENT_DEFINITION',
  FRAGMENT_SPREAD: 'FRAGMENT_SPREAD',
  INLINE_FRAGMENT: 'INLINE_FRAGMENT',
  VARIABLE_DEFINITION: 'VARIABLE_DEFINITION',
  // Type System Definitions
  SCHEMA: 'SCHEMA',
  SCALAR: 'SCALAR',
  OBJECT: 'OBJECT',
  FIELD_DEFINITION: 'FIELD_DEFINITION',
  ARGUMENT_DEFINITION: 'ARGUMENT_DEFINITION',
  INTERFACE: 'INTERFACE',
  UNION: 'UNION',
  ENUM: 'ENUM',
  ENUM_VALUE: 'ENUM_VALUE',
  INPUT_OBJECT: 'INPUT_OBJECT',
  INPUT_FIELD_DEFINITION: 'INPUT_FIELD_DEFINITION'
});
/**
 * The enum type representing the directive location values.
 */

/**
 * Produces the value of a block string from its parsed raw value, similar to
 * CoffeeScript's block string, Python's docstring trim or Ruby's strip_heredoc.
 *
 * This implements the GraphQL spec's BlockStringValue() static algorithm.
 *
 * @internal
 */
function dedentBlockStringValue(rawString) {
  // Expand a block string's raw value into independent lines.
  var lines = rawString.split(/\r\n|[\n\r]/g); // Remove common indentation from all lines but first.

  var commonIndent = getBlockStringIndentation(rawString);

  if (commonIndent !== 0) {
    for (var i = 1; i < lines.length; i++) {
      lines[i] = lines[i].slice(commonIndent);
    }
  } // Remove leading and trailing blank lines.


  var startLine = 0;

  while (startLine < lines.length && isBlank(lines[startLine])) {
    ++startLine;
  }

  var endLine = lines.length;

  while (endLine > startLine && isBlank(lines[endLine - 1])) {
    --endLine;
  } // Return a string of the lines joined with U+000A.


  return lines.slice(startLine, endLine).join('\n');
}

function isBlank(str) {
  for (var i = 0; i < str.length; ++i) {
    if (str[i] !== ' ' && str[i] !== '\t') {
      return false;
    }
  }

  return true;
}
/**
 * @internal
 */


function getBlockStringIndentation(value) {
  var _commonIndent;

  var isFirstLine = true;
  var isEmptyLine = true;
  var indent = 0;
  var commonIndent = null;

  for (var i = 0; i < value.length; ++i) {
    switch (value.charCodeAt(i)) {
      case 13:
        //  \r
        if (value.charCodeAt(i + 1) === 10) {
          ++i; // skip \r\n as one symbol
        }

      // falls through

      case 10:
        //  \n
        isFirstLine = false;
        isEmptyLine = true;
        indent = 0;
        break;

      case 9: //   \t

      case 32:
        //  <space>
        ++indent;
        break;

      default:
        if (isEmptyLine && !isFirstLine && (commonIndent === null || indent < commonIndent)) {
          commonIndent = indent;
        }

        isEmptyLine = false;
    }
  }

  return (_commonIndent = commonIndent) !== null && _commonIndent !== void 0 ? _commonIndent : 0;
}

/**
 * Given a Source object, creates a Lexer for that source.
 * A Lexer is a stateful stream generator in that every time
 * it is advanced, it returns the next token in the Source. Assuming the
 * source lexes, the final Token emitted by the lexer will be of kind
 * EOF, after which the lexer will repeatedly return the same EOF token
 * whenever called.
 */

var Lexer = /*#__PURE__*/function () {
  /**
   * The previously focused non-ignored token.
   */

  /**
   * The currently focused non-ignored token.
   */

  /**
   * The (1-indexed) line containing the current token.
   */

  /**
   * The character offset at which the current line begins.
   */
  function Lexer(source) {
    var startOfFileToken = new Token(TokenKind.SOF, 0, 0, 0, 0, null);
    this.source = source;
    this.lastToken = startOfFileToken;
    this.token = startOfFileToken;
    this.line = 1;
    this.lineStart = 0;
  }
  /**
   * Advances the token stream to the next non-ignored token.
   */


  var _proto = Lexer.prototype;

  _proto.advance = function advance() {
    this.lastToken = this.token;
    var token = this.token = this.lookahead();
    return token;
  }
  /**
   * Looks ahead and returns the next non-ignored token, but does not change
   * the state of Lexer.
   */
  ;

  _proto.lookahead = function lookahead() {
    var token = this.token;

    if (token.kind !== TokenKind.EOF) {
      do {
        var _token$next;

        // Note: next is only mutable during parsing, so we cast to allow this.
        token = (_token$next = token.next) !== null && _token$next !== void 0 ? _token$next : token.next = readToken(this, token);
      } while (token.kind === TokenKind.COMMENT);
    }

    return token;
  };

  return Lexer;
}();
/**
 * @internal
 */

function isPunctuatorTokenKind(kind) {
  return kind === TokenKind.BANG || kind === TokenKind.DOLLAR || kind === TokenKind.AMP || kind === TokenKind.PAREN_L || kind === TokenKind.PAREN_R || kind === TokenKind.SPREAD || kind === TokenKind.COLON || kind === TokenKind.EQUALS || kind === TokenKind.AT || kind === TokenKind.BRACKET_L || kind === TokenKind.BRACKET_R || kind === TokenKind.BRACE_L || kind === TokenKind.PIPE || kind === TokenKind.BRACE_R;
}

function printCharCode(code) {
  return (// NaN/undefined represents access beyond the end of the file.
    isNaN(code) ? TokenKind.EOF : // Trust JSON for ASCII.
    code < 0x007f ? JSON.stringify(String.fromCharCode(code)) : // Otherwise print the escaped form.
    "\"\\u".concat(('00' + code.toString(16).toUpperCase()).slice(-4), "\"")
  );
}
/**
 * Gets the next token from the source starting at the given position.
 *
 * This skips over whitespace until it finds the next lexable token, then lexes
 * punctuators immediately or calls the appropriate helper function for more
 * complicated tokens.
 */


function readToken(lexer, prev) {
  var source = lexer.source;
  var body = source.body;
  var bodyLength = body.length;
  var pos = prev.end;

  while (pos < bodyLength) {
    var code = body.charCodeAt(pos);
    var _line = lexer.line;

    var _col = 1 + pos - lexer.lineStart; // SourceCharacter


    switch (code) {
      case 0xfeff: // <BOM>

      case 9: //   \t

      case 32: //  <space>

      case 44:
        //  ,
        ++pos;
        continue;

      case 10:
        //  \n
        ++pos;
        ++lexer.line;
        lexer.lineStart = pos;
        continue;

      case 13:
        //  \r
        if (body.charCodeAt(pos + 1) === 10) {
          pos += 2;
        } else {
          ++pos;
        }

        ++lexer.line;
        lexer.lineStart = pos;
        continue;

      case 33:
        //  !
        return new Token(TokenKind.BANG, pos, pos + 1, _line, _col, prev);

      case 35:
        //  #
        return readComment(source, pos, _line, _col, prev);

      case 36:
        //  $
        return new Token(TokenKind.DOLLAR, pos, pos + 1, _line, _col, prev);

      case 38:
        //  &
        return new Token(TokenKind.AMP, pos, pos + 1, _line, _col, prev);

      case 40:
        //  (
        return new Token(TokenKind.PAREN_L, pos, pos + 1, _line, _col, prev);

      case 41:
        //  )
        return new Token(TokenKind.PAREN_R, pos, pos + 1, _line, _col, prev);

      case 46:
        //  .
        if (body.charCodeAt(pos + 1) === 46 && body.charCodeAt(pos + 2) === 46) {
          return new Token(TokenKind.SPREAD, pos, pos + 3, _line, _col, prev);
        }

        break;

      case 58:
        //  :
        return new Token(TokenKind.COLON, pos, pos + 1, _line, _col, prev);

      case 61:
        //  =
        return new Token(TokenKind.EQUALS, pos, pos + 1, _line, _col, prev);

      case 64:
        //  @
        return new Token(TokenKind.AT, pos, pos + 1, _line, _col, prev);

      case 91:
        //  [
        return new Token(TokenKind.BRACKET_L, pos, pos + 1, _line, _col, prev);

      case 93:
        //  ]
        return new Token(TokenKind.BRACKET_R, pos, pos + 1, _line, _col, prev);

      case 123:
        // {
        return new Token(TokenKind.BRACE_L, pos, pos + 1, _line, _col, prev);

      case 124:
        // |
        return new Token(TokenKind.PIPE, pos, pos + 1, _line, _col, prev);

      case 125:
        // }
        return new Token(TokenKind.BRACE_R, pos, pos + 1, _line, _col, prev);

      case 34:
        //  "
        if (body.charCodeAt(pos + 1) === 34 && body.charCodeAt(pos + 2) === 34) {
          return readBlockString(source, pos, _line, _col, prev, lexer);
        }

        return readString(source, pos, _line, _col, prev);

      case 45: //  -

      case 48: //  0

      case 49: //  1

      case 50: //  2

      case 51: //  3

      case 52: //  4

      case 53: //  5

      case 54: //  6

      case 55: //  7

      case 56: //  8

      case 57:
        //  9
        return readNumber(source, pos, code, _line, _col, prev);

      case 65: //  A

      case 66: //  B

      case 67: //  C

      case 68: //  D

      case 69: //  E

      case 70: //  F

      case 71: //  G

      case 72: //  H

      case 73: //  I

      case 74: //  J

      case 75: //  K

      case 76: //  L

      case 77: //  M

      case 78: //  N

      case 79: //  O

      case 80: //  P

      case 81: //  Q

      case 82: //  R

      case 83: //  S

      case 84: //  T

      case 85: //  U

      case 86: //  V

      case 87: //  W

      case 88: //  X

      case 89: //  Y

      case 90: //  Z

      case 95: //  _

      case 97: //  a

      case 98: //  b

      case 99: //  c

      case 100: // d

      case 101: // e

      case 102: // f

      case 103: // g

      case 104: // h

      case 105: // i

      case 106: // j

      case 107: // k

      case 108: // l

      case 109: // m

      case 110: // n

      case 111: // o

      case 112: // p

      case 113: // q

      case 114: // r

      case 115: // s

      case 116: // t

      case 117: // u

      case 118: // v

      case 119: // w

      case 120: // x

      case 121: // y

      case 122:
        // z
        return readName(source, pos, _line, _col, prev);
    }

    throw syntaxError(source, pos, unexpectedCharacterMessage(code));
  }

  var line = lexer.line;
  var col = 1 + pos - lexer.lineStart;
  return new Token(TokenKind.EOF, bodyLength, bodyLength, line, col, prev);
}
/**
 * Report a message that an unexpected character was encountered.
 */


function unexpectedCharacterMessage(code) {
  if (code < 0x0020 && code !== 0x0009 && code !== 0x000a && code !== 0x000d) {
    return "Cannot contain the invalid character ".concat(printCharCode(code), ".");
  }

  if (code === 39) {
    // '
    return 'Unexpected single quote character (\'), did you mean to use a double quote (")?';
  }

  return "Cannot parse the unexpected character ".concat(printCharCode(code), ".");
}
/**
 * Reads a comment token from the source file.
 *
 * #[\u0009\u0020-\uFFFF]*
 */


function readComment(source, start, line, col, prev) {
  var body = source.body;
  var code;
  var position = start;

  do {
    code = body.charCodeAt(++position);
  } while (!isNaN(code) && ( // SourceCharacter but not LineTerminator
  code > 0x001f || code === 0x0009));

  return new Token(TokenKind.COMMENT, start, position, line, col, prev, body.slice(start + 1, position));
}
/**
 * Reads a number token from the source file, either a float
 * or an int depending on whether a decimal point appears.
 *
 * Int:   -?(0|[1-9][0-9]*)
 * Float: -?(0|[1-9][0-9]*)(\.[0-9]+)?((E|e)(+|-)?[0-9]+)?
 */


function readNumber(source, start, firstCode, line, col, prev) {
  var body = source.body;
  var code = firstCode;
  var position = start;
  var isFloat = false;

  if (code === 45) {
    // -
    code = body.charCodeAt(++position);
  }

  if (code === 48) {
    // 0
    code = body.charCodeAt(++position);

    if (code >= 48 && code <= 57) {
      throw syntaxError(source, position, "Invalid number, unexpected digit after 0: ".concat(printCharCode(code), "."));
    }
  } else {
    position = readDigits(source, position, code);
    code = body.charCodeAt(position);
  }

  if (code === 46) {
    // .
    isFloat = true;
    code = body.charCodeAt(++position);
    position = readDigits(source, position, code);
    code = body.charCodeAt(position);
  }

  if (code === 69 || code === 101) {
    // E e
    isFloat = true;
    code = body.charCodeAt(++position);

    if (code === 43 || code === 45) {
      // + -
      code = body.charCodeAt(++position);
    }

    position = readDigits(source, position, code);
    code = body.charCodeAt(position);
  } // Numbers cannot be followed by . or NameStart


  if (code === 46 || isNameStart(code)) {
    throw syntaxError(source, position, "Invalid number, expected digit but got: ".concat(printCharCode(code), "."));
  }

  return new Token(isFloat ? TokenKind.FLOAT : TokenKind.INT, start, position, line, col, prev, body.slice(start, position));
}
/**
 * Returns the new position in the source after reading digits.
 */


function readDigits(source, start, firstCode) {
  var body = source.body;
  var position = start;
  var code = firstCode;

  if (code >= 48 && code <= 57) {
    // 0 - 9
    do {
      code = body.charCodeAt(++position);
    } while (code >= 48 && code <= 57); // 0 - 9


    return position;
  }

  throw syntaxError(source, position, "Invalid number, expected digit but got: ".concat(printCharCode(code), "."));
}
/**
 * Reads a string token from the source file.
 *
 * "([^"\\\u000A\u000D]|(\\(u[0-9a-fA-F]{4}|["\\/bfnrt])))*"
 */


function readString(source, start, line, col, prev) {
  var body = source.body;
  var position = start + 1;
  var chunkStart = position;
  var code = 0;
  var value = '';

  while (position < body.length && !isNaN(code = body.charCodeAt(position)) && // not LineTerminator
  code !== 0x000a && code !== 0x000d) {
    // Closing Quote (")
    if (code === 34) {
      value += body.slice(chunkStart, position);
      return new Token(TokenKind.STRING, start, position + 1, line, col, prev, value);
    } // SourceCharacter


    if (code < 0x0020 && code !== 0x0009) {
      throw syntaxError(source, position, "Invalid character within String: ".concat(printCharCode(code), "."));
    }

    ++position;

    if (code === 92) {
      // \
      value += body.slice(chunkStart, position - 1);
      code = body.charCodeAt(position);

      switch (code) {
        case 34:
          value += '"';
          break;

        case 47:
          value += '/';
          break;

        case 92:
          value += '\\';
          break;

        case 98:
          value += '\b';
          break;

        case 102:
          value += '\f';
          break;

        case 110:
          value += '\n';
          break;

        case 114:
          value += '\r';
          break;

        case 116:
          value += '\t';
          break;

        case 117:
          {
            // uXXXX
            var charCode = uniCharCode(body.charCodeAt(position + 1), body.charCodeAt(position + 2), body.charCodeAt(position + 3), body.charCodeAt(position + 4));

            if (charCode < 0) {
              var invalidSequence = body.slice(position + 1, position + 5);
              throw syntaxError(source, position, "Invalid character escape sequence: \\u".concat(invalidSequence, "."));
            }

            value += String.fromCharCode(charCode);
            position += 4;
            break;
          }

        default:
          throw syntaxError(source, position, "Invalid character escape sequence: \\".concat(String.fromCharCode(code), "."));
      }

      ++position;
      chunkStart = position;
    }
  }

  throw syntaxError(source, position, 'Unterminated string.');
}
/**
 * Reads a block string token from the source file.
 *
 * """("?"?(\\"""|\\(?!=""")|[^"\\]))*"""
 */


function readBlockString(source, start, line, col, prev, lexer) {
  var body = source.body;
  var position = start + 3;
  var chunkStart = position;
  var code = 0;
  var rawValue = '';

  while (position < body.length && !isNaN(code = body.charCodeAt(position))) {
    // Closing Triple-Quote (""")
    if (code === 34 && body.charCodeAt(position + 1) === 34 && body.charCodeAt(position + 2) === 34) {
      rawValue += body.slice(chunkStart, position);
      return new Token(TokenKind.BLOCK_STRING, start, position + 3, line, col, prev, dedentBlockStringValue(rawValue));
    } // SourceCharacter


    if (code < 0x0020 && code !== 0x0009 && code !== 0x000a && code !== 0x000d) {
      throw syntaxError(source, position, "Invalid character within String: ".concat(printCharCode(code), "."));
    }

    if (code === 10) {
      // new line
      ++position;
      ++lexer.line;
      lexer.lineStart = position;
    } else if (code === 13) {
      // carriage return
      if (body.charCodeAt(position + 1) === 10) {
        position += 2;
      } else {
        ++position;
      }

      ++lexer.line;
      lexer.lineStart = position;
    } else if ( // Escape Triple-Quote (\""")
    code === 92 && body.charCodeAt(position + 1) === 34 && body.charCodeAt(position + 2) === 34 && body.charCodeAt(position + 3) === 34) {
      rawValue += body.slice(chunkStart, position) + '"""';
      position += 4;
      chunkStart = position;
    } else {
      ++position;
    }
  }

  throw syntaxError(source, position, 'Unterminated string.');
}
/**
 * Converts four hexadecimal chars to the integer that the
 * string represents. For example, uniCharCode('0','0','0','f')
 * will return 15, and uniCharCode('0','0','f','f') returns 255.
 *
 * Returns a negative number on error, if a char was invalid.
 *
 * This is implemented by noting that char2hex() returns -1 on error,
 * which means the result of ORing the char2hex() will also be negative.
 */


function uniCharCode(a, b, c, d) {
  return char2hex(a) << 12 | char2hex(b) << 8 | char2hex(c) << 4 | char2hex(d);
}
/**
 * Converts a hex character to its integer value.
 * '0' becomes 0, '9' becomes 9
 * 'A' becomes 10, 'F' becomes 15
 * 'a' becomes 10, 'f' becomes 15
 *
 * Returns -1 on error.
 */


function char2hex(a) {
  return a >= 48 && a <= 57 ? a - 48 // 0-9
  : a >= 65 && a <= 70 ? a - 55 // A-F
  : a >= 97 && a <= 102 ? a - 87 // a-f
  : -1;
}
/**
 * Reads an alphanumeric + underscore name from the source.
 *
 * [_A-Za-z][_0-9A-Za-z]*
 */


function readName(source, start, line, col, prev) {
  var body = source.body;
  var bodyLength = body.length;
  var position = start + 1;
  var code = 0;

  while (position !== bodyLength && !isNaN(code = body.charCodeAt(position)) && (code === 95 || // _
  code >= 48 && code <= 57 || // 0-9
  code >= 65 && code <= 90 || // A-Z
  code >= 97 && code <= 122) // a-z
  ) {
    ++position;
  }

  return new Token(TokenKind.NAME, start, position, line, col, prev, body.slice(start, position));
} // _ A-Z a-z


function isNameStart(code) {
  return code === 95 || code >= 65 && code <= 90 || code >= 97 && code <= 122;
}

/**
 * Configuration options to control parser behavior
 */

/**
 * Given a GraphQL source, parses it into a Document.
 * Throws GraphQLError if a syntax error is encountered.
 */
function parse(source, options) {
  var parser = new Parser(source, options);
  return parser.parseDocument();
}
/**
 * This class is exported only to assist people in implementing their own parsers
 * without duplicating too much code and should be used only as last resort for cases
 * such as experimental syntax or if certain features could not be contributed upstream.
 *
 * It is still part of the internal API and is versioned, so any changes to it are never
 * considered breaking changes. If you still need to support multiple versions of the
 * library, please use the `versionInfo` variable for version detection.
 *
 * @internal
 */

var Parser = /*#__PURE__*/function () {
  function Parser(source, options) {
    var sourceObj = isSource(source) ? source : new Source(source);
    this._lexer = new Lexer(sourceObj);
    this._options = options;
  }
  /**
   * Converts a name lex token into a name parse node.
   */


  var _proto = Parser.prototype;

  _proto.parseName = function parseName() {
    var token = this.expectToken(TokenKind.NAME);
    return {
      kind: Kind.NAME,
      value: token.value,
      loc: this.loc(token)
    };
  } // Implements the parsing rules in the Document section.

  /**
   * Document : Definition+
   */
  ;

  _proto.parseDocument = function parseDocument() {
    var start = this._lexer.token;
    return {
      kind: Kind.DOCUMENT,
      definitions: this.many(TokenKind.SOF, this.parseDefinition, TokenKind.EOF),
      loc: this.loc(start)
    };
  }
  /**
   * Definition :
   *   - ExecutableDefinition
   *   - TypeSystemDefinition
   *   - TypeSystemExtension
   *
   * ExecutableDefinition :
   *   - OperationDefinition
   *   - FragmentDefinition
   */
  ;

  _proto.parseDefinition = function parseDefinition() {
    if (this.peek(TokenKind.NAME)) {
      switch (this._lexer.token.value) {
        case 'query':
        case 'mutation':
        case 'subscription':
          return this.parseOperationDefinition();

        case 'fragment':
          return this.parseFragmentDefinition();

        case 'schema':
        case 'scalar':
        case 'type':
        case 'interface':
        case 'union':
        case 'enum':
        case 'input':
        case 'directive':
          return this.parseTypeSystemDefinition();

        case 'extend':
          return this.parseTypeSystemExtension();
      }
    } else if (this.peek(TokenKind.BRACE_L)) {
      return this.parseOperationDefinition();
    } else if (this.peekDescription()) {
      return this.parseTypeSystemDefinition();
    }

    throw this.unexpected();
  } // Implements the parsing rules in the Operations section.

  /**
   * OperationDefinition :
   *  - SelectionSet
   *  - OperationType Name? VariableDefinitions? Directives? SelectionSet
   */
  ;

  _proto.parseOperationDefinition = function parseOperationDefinition() {
    var start = this._lexer.token;

    if (this.peek(TokenKind.BRACE_L)) {
      return {
        kind: Kind.OPERATION_DEFINITION,
        operation: 'query',
        name: undefined,
        variableDefinitions: [],
        directives: [],
        selectionSet: this.parseSelectionSet(),
        loc: this.loc(start)
      };
    }

    var operation = this.parseOperationType();
    var name;

    if (this.peek(TokenKind.NAME)) {
      name = this.parseName();
    }

    return {
      kind: Kind.OPERATION_DEFINITION,
      operation: operation,
      name: name,
      variableDefinitions: this.parseVariableDefinitions(),
      directives: this.parseDirectives(false),
      selectionSet: this.parseSelectionSet(),
      loc: this.loc(start)
    };
  }
  /**
   * OperationType : one of query mutation subscription
   */
  ;

  _proto.parseOperationType = function parseOperationType() {
    var operationToken = this.expectToken(TokenKind.NAME);

    switch (operationToken.value) {
      case 'query':
        return 'query';

      case 'mutation':
        return 'mutation';

      case 'subscription':
        return 'subscription';
    }

    throw this.unexpected(operationToken);
  }
  /**
   * VariableDefinitions : ( VariableDefinition+ )
   */
  ;

  _proto.parseVariableDefinitions = function parseVariableDefinitions() {
    return this.optionalMany(TokenKind.PAREN_L, this.parseVariableDefinition, TokenKind.PAREN_R);
  }
  /**
   * VariableDefinition : Variable : Type DefaultValue? Directives[Const]?
   */
  ;

  _proto.parseVariableDefinition = function parseVariableDefinition() {
    var start = this._lexer.token;
    return {
      kind: Kind.VARIABLE_DEFINITION,
      variable: this.parseVariable(),
      type: (this.expectToken(TokenKind.COLON), this.parseTypeReference()),
      defaultValue: this.expectOptionalToken(TokenKind.EQUALS) ? this.parseValueLiteral(true) : undefined,
      directives: this.parseDirectives(true),
      loc: this.loc(start)
    };
  }
  /**
   * Variable : $ Name
   */
  ;

  _proto.parseVariable = function parseVariable() {
    var start = this._lexer.token;
    this.expectToken(TokenKind.DOLLAR);
    return {
      kind: Kind.VARIABLE,
      name: this.parseName(),
      loc: this.loc(start)
    };
  }
  /**
   * SelectionSet : { Selection+ }
   */
  ;

  _proto.parseSelectionSet = function parseSelectionSet() {
    var start = this._lexer.token;
    return {
      kind: Kind.SELECTION_SET,
      selections: this.many(TokenKind.BRACE_L, this.parseSelection, TokenKind.BRACE_R),
      loc: this.loc(start)
    };
  }
  /**
   * Selection :
   *   - Field
   *   - FragmentSpread
   *   - InlineFragment
   */
  ;

  _proto.parseSelection = function parseSelection() {
    return this.peek(TokenKind.SPREAD) ? this.parseFragment() : this.parseField();
  }
  /**
   * Field : Alias? Name Arguments? Directives? SelectionSet?
   *
   * Alias : Name :
   */
  ;

  _proto.parseField = function parseField() {
    var start = this._lexer.token;
    var nameOrAlias = this.parseName();
    var alias;
    var name;

    if (this.expectOptionalToken(TokenKind.COLON)) {
      alias = nameOrAlias;
      name = this.parseName();
    } else {
      name = nameOrAlias;
    }

    return {
      kind: Kind.FIELD,
      alias: alias,
      name: name,
      arguments: this.parseArguments(false),
      directives: this.parseDirectives(false),
      selectionSet: this.peek(TokenKind.BRACE_L) ? this.parseSelectionSet() : undefined,
      loc: this.loc(start)
    };
  }
  /**
   * Arguments[Const] : ( Argument[?Const]+ )
   */
  ;

  _proto.parseArguments = function parseArguments(isConst) {
    var item = isConst ? this.parseConstArgument : this.parseArgument;
    return this.optionalMany(TokenKind.PAREN_L, item, TokenKind.PAREN_R);
  }
  /**
   * Argument[Const] : Name : Value[?Const]
   */
  ;

  _proto.parseArgument = function parseArgument() {
    var start = this._lexer.token;
    var name = this.parseName();
    this.expectToken(TokenKind.COLON);
    return {
      kind: Kind.ARGUMENT,
      name: name,
      value: this.parseValueLiteral(false),
      loc: this.loc(start)
    };
  };

  _proto.parseConstArgument = function parseConstArgument() {
    var start = this._lexer.token;
    return {
      kind: Kind.ARGUMENT,
      name: this.parseName(),
      value: (this.expectToken(TokenKind.COLON), this.parseValueLiteral(true)),
      loc: this.loc(start)
    };
  } // Implements the parsing rules in the Fragments section.

  /**
   * Corresponds to both FragmentSpread and InlineFragment in the spec.
   *
   * FragmentSpread : ... FragmentName Directives?
   *
   * InlineFragment : ... TypeCondition? Directives? SelectionSet
   */
  ;

  _proto.parseFragment = function parseFragment() {
    var start = this._lexer.token;
    this.expectToken(TokenKind.SPREAD);
    var hasTypeCondition = this.expectOptionalKeyword('on');

    if (!hasTypeCondition && this.peek(TokenKind.NAME)) {
      return {
        kind: Kind.FRAGMENT_SPREAD,
        name: this.parseFragmentName(),
        directives: this.parseDirectives(false),
        loc: this.loc(start)
      };
    }

    return {
      kind: Kind.INLINE_FRAGMENT,
      typeCondition: hasTypeCondition ? this.parseNamedType() : undefined,
      directives: this.parseDirectives(false),
      selectionSet: this.parseSelectionSet(),
      loc: this.loc(start)
    };
  }
  /**
   * FragmentDefinition :
   *   - fragment FragmentName on TypeCondition Directives? SelectionSet
   *
   * TypeCondition : NamedType
   */
  ;

  _proto.parseFragmentDefinition = function parseFragmentDefinition() {
    var _this$_options;

    var start = this._lexer.token;
    this.expectKeyword('fragment'); // Experimental support for defining variables within fragments changes
    // the grammar of FragmentDefinition:
    //   - fragment FragmentName VariableDefinitions? on TypeCondition Directives? SelectionSet

    if (((_this$_options = this._options) === null || _this$_options === void 0 ? void 0 : _this$_options.experimentalFragmentVariables) === true) {
      return {
        kind: Kind.FRAGMENT_DEFINITION,
        name: this.parseFragmentName(),
        variableDefinitions: this.parseVariableDefinitions(),
        typeCondition: (this.expectKeyword('on'), this.parseNamedType()),
        directives: this.parseDirectives(false),
        selectionSet: this.parseSelectionSet(),
        loc: this.loc(start)
      };
    }

    return {
      kind: Kind.FRAGMENT_DEFINITION,
      name: this.parseFragmentName(),
      typeCondition: (this.expectKeyword('on'), this.parseNamedType()),
      directives: this.parseDirectives(false),
      selectionSet: this.parseSelectionSet(),
      loc: this.loc(start)
    };
  }
  /**
   * FragmentName : Name but not `on`
   */
  ;

  _proto.parseFragmentName = function parseFragmentName() {
    if (this._lexer.token.value === 'on') {
      throw this.unexpected();
    }

    return this.parseName();
  } // Implements the parsing rules in the Values section.

  /**
   * Value[Const] :
   *   - [~Const] Variable
   *   - IntValue
   *   - FloatValue
   *   - StringValue
   *   - BooleanValue
   *   - NullValue
   *   - EnumValue
   *   - ListValue[?Const]
   *   - ObjectValue[?Const]
   *
   * BooleanValue : one of `true` `false`
   *
   * NullValue : `null`
   *
   * EnumValue : Name but not `true`, `false` or `null`
   */
  ;

  _proto.parseValueLiteral = function parseValueLiteral(isConst) {
    var token = this._lexer.token;

    switch (token.kind) {
      case TokenKind.BRACKET_L:
        return this.parseList(isConst);

      case TokenKind.BRACE_L:
        return this.parseObject(isConst);

      case TokenKind.INT:
        this._lexer.advance();

        return {
          kind: Kind.INT,
          value: token.value,
          loc: this.loc(token)
        };

      case TokenKind.FLOAT:
        this._lexer.advance();

        return {
          kind: Kind.FLOAT,
          value: token.value,
          loc: this.loc(token)
        };

      case TokenKind.STRING:
      case TokenKind.BLOCK_STRING:
        return this.parseStringLiteral();

      case TokenKind.NAME:
        this._lexer.advance();

        switch (token.value) {
          case 'true':
            return {
              kind: Kind.BOOLEAN,
              value: true,
              loc: this.loc(token)
            };

          case 'false':
            return {
              kind: Kind.BOOLEAN,
              value: false,
              loc: this.loc(token)
            };

          case 'null':
            return {
              kind: Kind.NULL,
              loc: this.loc(token)
            };

          default:
            return {
              kind: Kind.ENUM,
              value: token.value,
              loc: this.loc(token)
            };
        }

      case TokenKind.DOLLAR:
        if (!isConst) {
          return this.parseVariable();
        }

        break;
    }

    throw this.unexpected();
  };

  _proto.parseStringLiteral = function parseStringLiteral() {
    var token = this._lexer.token;

    this._lexer.advance();

    return {
      kind: Kind.STRING,
      value: token.value,
      block: token.kind === TokenKind.BLOCK_STRING,
      loc: this.loc(token)
    };
  }
  /**
   * ListValue[Const] :
   *   - [ ]
   *   - [ Value[?Const]+ ]
   */
  ;

  _proto.parseList = function parseList(isConst) {
    var _this = this;

    var start = this._lexer.token;

    var item = function item() {
      return _this.parseValueLiteral(isConst);
    };

    return {
      kind: Kind.LIST,
      values: this.any(TokenKind.BRACKET_L, item, TokenKind.BRACKET_R),
      loc: this.loc(start)
    };
  }
  /**
   * ObjectValue[Const] :
   *   - { }
   *   - { ObjectField[?Const]+ }
   */
  ;

  _proto.parseObject = function parseObject(isConst) {
    var _this2 = this;

    var start = this._lexer.token;

    var item = function item() {
      return _this2.parseObjectField(isConst);
    };

    return {
      kind: Kind.OBJECT,
      fields: this.any(TokenKind.BRACE_L, item, TokenKind.BRACE_R),
      loc: this.loc(start)
    };
  }
  /**
   * ObjectField[Const] : Name : Value[?Const]
   */
  ;

  _proto.parseObjectField = function parseObjectField(isConst) {
    var start = this._lexer.token;
    var name = this.parseName();
    this.expectToken(TokenKind.COLON);
    return {
      kind: Kind.OBJECT_FIELD,
      name: name,
      value: this.parseValueLiteral(isConst),
      loc: this.loc(start)
    };
  } // Implements the parsing rules in the Directives section.

  /**
   * Directives[Const] : Directive[?Const]+
   */
  ;

  _proto.parseDirectives = function parseDirectives(isConst) {
    var directives = [];

    while (this.peek(TokenKind.AT)) {
      directives.push(this.parseDirective(isConst));
    }

    return directives;
  }
  /**
   * Directive[Const] : @ Name Arguments[?Const]?
   */
  ;

  _proto.parseDirective = function parseDirective(isConst) {
    var start = this._lexer.token;
    this.expectToken(TokenKind.AT);
    return {
      kind: Kind.DIRECTIVE,
      name: this.parseName(),
      arguments: this.parseArguments(isConst),
      loc: this.loc(start)
    };
  } // Implements the parsing rules in the Types section.

  /**
   * Type :
   *   - NamedType
   *   - ListType
   *   - NonNullType
   */
  ;

  _proto.parseTypeReference = function parseTypeReference() {
    var start = this._lexer.token;
    var type;

    if (this.expectOptionalToken(TokenKind.BRACKET_L)) {
      type = this.parseTypeReference();
      this.expectToken(TokenKind.BRACKET_R);
      type = {
        kind: Kind.LIST_TYPE,
        type: type,
        loc: this.loc(start)
      };
    } else {
      type = this.parseNamedType();
    }

    if (this.expectOptionalToken(TokenKind.BANG)) {
      return {
        kind: Kind.NON_NULL_TYPE,
        type: type,
        loc: this.loc(start)
      };
    }

    return type;
  }
  /**
   * NamedType : Name
   */
  ;

  _proto.parseNamedType = function parseNamedType() {
    var start = this._lexer.token;
    return {
      kind: Kind.NAMED_TYPE,
      name: this.parseName(),
      loc: this.loc(start)
    };
  } // Implements the parsing rules in the Type Definition section.

  /**
   * TypeSystemDefinition :
   *   - SchemaDefinition
   *   - TypeDefinition
   *   - DirectiveDefinition
   *
   * TypeDefinition :
   *   - ScalarTypeDefinition
   *   - ObjectTypeDefinition
   *   - InterfaceTypeDefinition
   *   - UnionTypeDefinition
   *   - EnumTypeDefinition
   *   - InputObjectTypeDefinition
   */
  ;

  _proto.parseTypeSystemDefinition = function parseTypeSystemDefinition() {
    // Many definitions begin with a description and require a lookahead.
    var keywordToken = this.peekDescription() ? this._lexer.lookahead() : this._lexer.token;

    if (keywordToken.kind === TokenKind.NAME) {
      switch (keywordToken.value) {
        case 'schema':
          return this.parseSchemaDefinition();

        case 'scalar':
          return this.parseScalarTypeDefinition();

        case 'type':
          return this.parseObjectTypeDefinition();

        case 'interface':
          return this.parseInterfaceTypeDefinition();

        case 'union':
          return this.parseUnionTypeDefinition();

        case 'enum':
          return this.parseEnumTypeDefinition();

        case 'input':
          return this.parseInputObjectTypeDefinition();

        case 'directive':
          return this.parseDirectiveDefinition();
      }
    }

    throw this.unexpected(keywordToken);
  };

  _proto.peekDescription = function peekDescription() {
    return this.peek(TokenKind.STRING) || this.peek(TokenKind.BLOCK_STRING);
  }
  /**
   * Description : StringValue
   */
  ;

  _proto.parseDescription = function parseDescription() {
    if (this.peekDescription()) {
      return this.parseStringLiteral();
    }
  }
  /**
   * SchemaDefinition : Description? schema Directives[Const]? { OperationTypeDefinition+ }
   */
  ;

  _proto.parseSchemaDefinition = function parseSchemaDefinition() {
    var start = this._lexer.token;
    var description = this.parseDescription();
    this.expectKeyword('schema');
    var directives = this.parseDirectives(true);
    var operationTypes = this.many(TokenKind.BRACE_L, this.parseOperationTypeDefinition, TokenKind.BRACE_R);
    return {
      kind: Kind.SCHEMA_DEFINITION,
      description: description,
      directives: directives,
      operationTypes: operationTypes,
      loc: this.loc(start)
    };
  }
  /**
   * OperationTypeDefinition : OperationType : NamedType
   */
  ;

  _proto.parseOperationTypeDefinition = function parseOperationTypeDefinition() {
    var start = this._lexer.token;
    var operation = this.parseOperationType();
    this.expectToken(TokenKind.COLON);
    var type = this.parseNamedType();
    return {
      kind: Kind.OPERATION_TYPE_DEFINITION,
      operation: operation,
      type: type,
      loc: this.loc(start)
    };
  }
  /**
   * ScalarTypeDefinition : Description? scalar Name Directives[Const]?
   */
  ;

  _proto.parseScalarTypeDefinition = function parseScalarTypeDefinition() {
    var start = this._lexer.token;
    var description = this.parseDescription();
    this.expectKeyword('scalar');
    var name = this.parseName();
    var directives = this.parseDirectives(true);
    return {
      kind: Kind.SCALAR_TYPE_DEFINITION,
      description: description,
      name: name,
      directives: directives,
      loc: this.loc(start)
    };
  }
  /**
   * ObjectTypeDefinition :
   *   Description?
   *   type Name ImplementsInterfaces? Directives[Const]? FieldsDefinition?
   */
  ;

  _proto.parseObjectTypeDefinition = function parseObjectTypeDefinition() {
    var start = this._lexer.token;
    var description = this.parseDescription();
    this.expectKeyword('type');
    var name = this.parseName();
    var interfaces = this.parseImplementsInterfaces();
    var directives = this.parseDirectives(true);
    var fields = this.parseFieldsDefinition();
    return {
      kind: Kind.OBJECT_TYPE_DEFINITION,
      description: description,
      name: name,
      interfaces: interfaces,
      directives: directives,
      fields: fields,
      loc: this.loc(start)
    };
  }
  /**
   * ImplementsInterfaces :
   *   - implements `&`? NamedType
   *   - ImplementsInterfaces & NamedType
   */
  ;

  _proto.parseImplementsInterfaces = function parseImplementsInterfaces() {
    var _this$_options2;

    if (!this.expectOptionalKeyword('implements')) {
      return [];
    }

    if (((_this$_options2 = this._options) === null || _this$_options2 === void 0 ? void 0 : _this$_options2.allowLegacySDLImplementsInterfaces) === true) {
      var types = []; // Optional leading ampersand

      this.expectOptionalToken(TokenKind.AMP);

      do {
        types.push(this.parseNamedType());
      } while (this.expectOptionalToken(TokenKind.AMP) || this.peek(TokenKind.NAME));

      return types;
    }

    return this.delimitedMany(TokenKind.AMP, this.parseNamedType);
  }
  /**
   * FieldsDefinition : { FieldDefinition+ }
   */
  ;

  _proto.parseFieldsDefinition = function parseFieldsDefinition() {
    var _this$_options3;

    // Legacy support for the SDL?
    if (((_this$_options3 = this._options) === null || _this$_options3 === void 0 ? void 0 : _this$_options3.allowLegacySDLEmptyFields) === true && this.peek(TokenKind.BRACE_L) && this._lexer.lookahead().kind === TokenKind.BRACE_R) {
      this._lexer.advance();

      this._lexer.advance();

      return [];
    }

    return this.optionalMany(TokenKind.BRACE_L, this.parseFieldDefinition, TokenKind.BRACE_R);
  }
  /**
   * FieldDefinition :
   *   - Description? Name ArgumentsDefinition? : Type Directives[Const]?
   */
  ;

  _proto.parseFieldDefinition = function parseFieldDefinition() {
    var start = this._lexer.token;
    var description = this.parseDescription();
    var name = this.parseName();
    var args = this.parseArgumentDefs();
    this.expectToken(TokenKind.COLON);
    var type = this.parseTypeReference();
    var directives = this.parseDirectives(true);
    return {
      kind: Kind.FIELD_DEFINITION,
      description: description,
      name: name,
      arguments: args,
      type: type,
      directives: directives,
      loc: this.loc(start)
    };
  }
  /**
   * ArgumentsDefinition : ( InputValueDefinition+ )
   */
  ;

  _proto.parseArgumentDefs = function parseArgumentDefs() {
    return this.optionalMany(TokenKind.PAREN_L, this.parseInputValueDef, TokenKind.PAREN_R);
  }
  /**
   * InputValueDefinition :
   *   - Description? Name : Type DefaultValue? Directives[Const]?
   */
  ;

  _proto.parseInputValueDef = function parseInputValueDef() {
    var start = this._lexer.token;
    var description = this.parseDescription();
    var name = this.parseName();
    this.expectToken(TokenKind.COLON);
    var type = this.parseTypeReference();
    var defaultValue;

    if (this.expectOptionalToken(TokenKind.EQUALS)) {
      defaultValue = this.parseValueLiteral(true);
    }

    var directives = this.parseDirectives(true);
    return {
      kind: Kind.INPUT_VALUE_DEFINITION,
      description: description,
      name: name,
      type: type,
      defaultValue: defaultValue,
      directives: directives,
      loc: this.loc(start)
    };
  }
  /**
   * InterfaceTypeDefinition :
   *   - Description? interface Name Directives[Const]? FieldsDefinition?
   */
  ;

  _proto.parseInterfaceTypeDefinition = function parseInterfaceTypeDefinition() {
    var start = this._lexer.token;
    var description = this.parseDescription();
    this.expectKeyword('interface');
    var name = this.parseName();
    var interfaces = this.parseImplementsInterfaces();
    var directives = this.parseDirectives(true);
    var fields = this.parseFieldsDefinition();
    return {
      kind: Kind.INTERFACE_TYPE_DEFINITION,
      description: description,
      name: name,
      interfaces: interfaces,
      directives: directives,
      fields: fields,
      loc: this.loc(start)
    };
  }
  /**
   * UnionTypeDefinition :
   *   - Description? union Name Directives[Const]? UnionMemberTypes?
   */
  ;

  _proto.parseUnionTypeDefinition = function parseUnionTypeDefinition() {
    var start = this._lexer.token;
    var description = this.parseDescription();
    this.expectKeyword('union');
    var name = this.parseName();
    var directives = this.parseDirectives(true);
    var types = this.parseUnionMemberTypes();
    return {
      kind: Kind.UNION_TYPE_DEFINITION,
      description: description,
      name: name,
      directives: directives,
      types: types,
      loc: this.loc(start)
    };
  }
  /**
   * UnionMemberTypes :
   *   - = `|`? NamedType
   *   - UnionMemberTypes | NamedType
   */
  ;

  _proto.parseUnionMemberTypes = function parseUnionMemberTypes() {
    return this.expectOptionalToken(TokenKind.EQUALS) ? this.delimitedMany(TokenKind.PIPE, this.parseNamedType) : [];
  }
  /**
   * EnumTypeDefinition :
   *   - Description? enum Name Directives[Const]? EnumValuesDefinition?
   */
  ;

  _proto.parseEnumTypeDefinition = function parseEnumTypeDefinition() {
    var start = this._lexer.token;
    var description = this.parseDescription();
    this.expectKeyword('enum');
    var name = this.parseName();
    var directives = this.parseDirectives(true);
    var values = this.parseEnumValuesDefinition();
    return {
      kind: Kind.ENUM_TYPE_DEFINITION,
      description: description,
      name: name,
      directives: directives,
      values: values,
      loc: this.loc(start)
    };
  }
  /**
   * EnumValuesDefinition : { EnumValueDefinition+ }
   */
  ;

  _proto.parseEnumValuesDefinition = function parseEnumValuesDefinition() {
    return this.optionalMany(TokenKind.BRACE_L, this.parseEnumValueDefinition, TokenKind.BRACE_R);
  }
  /**
   * EnumValueDefinition : Description? EnumValue Directives[Const]?
   *
   * EnumValue : Name
   */
  ;

  _proto.parseEnumValueDefinition = function parseEnumValueDefinition() {
    var start = this._lexer.token;
    var description = this.parseDescription();
    var name = this.parseName();
    var directives = this.parseDirectives(true);
    return {
      kind: Kind.ENUM_VALUE_DEFINITION,
      description: description,
      name: name,
      directives: directives,
      loc: this.loc(start)
    };
  }
  /**
   * InputObjectTypeDefinition :
   *   - Description? input Name Directives[Const]? InputFieldsDefinition?
   */
  ;

  _proto.parseInputObjectTypeDefinition = function parseInputObjectTypeDefinition() {
    var start = this._lexer.token;
    var description = this.parseDescription();
    this.expectKeyword('input');
    var name = this.parseName();
    var directives = this.parseDirectives(true);
    var fields = this.parseInputFieldsDefinition();
    return {
      kind: Kind.INPUT_OBJECT_TYPE_DEFINITION,
      description: description,
      name: name,
      directives: directives,
      fields: fields,
      loc: this.loc(start)
    };
  }
  /**
   * InputFieldsDefinition : { InputValueDefinition+ }
   */
  ;

  _proto.parseInputFieldsDefinition = function parseInputFieldsDefinition() {
    return this.optionalMany(TokenKind.BRACE_L, this.parseInputValueDef, TokenKind.BRACE_R);
  }
  /**
   * TypeSystemExtension :
   *   - SchemaExtension
   *   - TypeExtension
   *
   * TypeExtension :
   *   - ScalarTypeExtension
   *   - ObjectTypeExtension
   *   - InterfaceTypeExtension
   *   - UnionTypeExtension
   *   - EnumTypeExtension
   *   - InputObjectTypeDefinition
   */
  ;

  _proto.parseTypeSystemExtension = function parseTypeSystemExtension() {
    var keywordToken = this._lexer.lookahead();

    if (keywordToken.kind === TokenKind.NAME) {
      switch (keywordToken.value) {
        case 'schema':
          return this.parseSchemaExtension();

        case 'scalar':
          return this.parseScalarTypeExtension();

        case 'type':
          return this.parseObjectTypeExtension();

        case 'interface':
          return this.parseInterfaceTypeExtension();

        case 'union':
          return this.parseUnionTypeExtension();

        case 'enum':
          return this.parseEnumTypeExtension();

        case 'input':
          return this.parseInputObjectTypeExtension();
      }
    }

    throw this.unexpected(keywordToken);
  }
  /**
   * SchemaExtension :
   *  - extend schema Directives[Const]? { OperationTypeDefinition+ }
   *  - extend schema Directives[Const]
   */
  ;

  _proto.parseSchemaExtension = function parseSchemaExtension() {
    var start = this._lexer.token;
    this.expectKeyword('extend');
    this.expectKeyword('schema');
    var directives = this.parseDirectives(true);
    var operationTypes = this.optionalMany(TokenKind.BRACE_L, this.parseOperationTypeDefinition, TokenKind.BRACE_R);

    if (directives.length === 0 && operationTypes.length === 0) {
      throw this.unexpected();
    }

    return {
      kind: Kind.SCHEMA_EXTENSION,
      directives: directives,
      operationTypes: operationTypes,
      loc: this.loc(start)
    };
  }
  /**
   * ScalarTypeExtension :
   *   - extend scalar Name Directives[Const]
   */
  ;

  _proto.parseScalarTypeExtension = function parseScalarTypeExtension() {
    var start = this._lexer.token;
    this.expectKeyword('extend');
    this.expectKeyword('scalar');
    var name = this.parseName();
    var directives = this.parseDirectives(true);

    if (directives.length === 0) {
      throw this.unexpected();
    }

    return {
      kind: Kind.SCALAR_TYPE_EXTENSION,
      name: name,
      directives: directives,
      loc: this.loc(start)
    };
  }
  /**
   * ObjectTypeExtension :
   *  - extend type Name ImplementsInterfaces? Directives[Const]? FieldsDefinition
   *  - extend type Name ImplementsInterfaces? Directives[Const]
   *  - extend type Name ImplementsInterfaces
   */
  ;

  _proto.parseObjectTypeExtension = function parseObjectTypeExtension() {
    var start = this._lexer.token;
    this.expectKeyword('extend');
    this.expectKeyword('type');
    var name = this.parseName();
    var interfaces = this.parseImplementsInterfaces();
    var directives = this.parseDirectives(true);
    var fields = this.parseFieldsDefinition();

    if (interfaces.length === 0 && directives.length === 0 && fields.length === 0) {
      throw this.unexpected();
    }

    return {
      kind: Kind.OBJECT_TYPE_EXTENSION,
      name: name,
      interfaces: interfaces,
      directives: directives,
      fields: fields,
      loc: this.loc(start)
    };
  }
  /**
   * InterfaceTypeExtension :
   *  - extend interface Name ImplementsInterfaces? Directives[Const]? FieldsDefinition
   *  - extend interface Name ImplementsInterfaces? Directives[Const]
   *  - extend interface Name ImplementsInterfaces
   */
  ;

  _proto.parseInterfaceTypeExtension = function parseInterfaceTypeExtension() {
    var start = this._lexer.token;
    this.expectKeyword('extend');
    this.expectKeyword('interface');
    var name = this.parseName();
    var interfaces = this.parseImplementsInterfaces();
    var directives = this.parseDirectives(true);
    var fields = this.parseFieldsDefinition();

    if (interfaces.length === 0 && directives.length === 0 && fields.length === 0) {
      throw this.unexpected();
    }

    return {
      kind: Kind.INTERFACE_TYPE_EXTENSION,
      name: name,
      interfaces: interfaces,
      directives: directives,
      fields: fields,
      loc: this.loc(start)
    };
  }
  /**
   * UnionTypeExtension :
   *   - extend union Name Directives[Const]? UnionMemberTypes
   *   - extend union Name Directives[Const]
   */
  ;

  _proto.parseUnionTypeExtension = function parseUnionTypeExtension() {
    var start = this._lexer.token;
    this.expectKeyword('extend');
    this.expectKeyword('union');
    var name = this.parseName();
    var directives = this.parseDirectives(true);
    var types = this.parseUnionMemberTypes();

    if (directives.length === 0 && types.length === 0) {
      throw this.unexpected();
    }

    return {
      kind: Kind.UNION_TYPE_EXTENSION,
      name: name,
      directives: directives,
      types: types,
      loc: this.loc(start)
    };
  }
  /**
   * EnumTypeExtension :
   *   - extend enum Name Directives[Const]? EnumValuesDefinition
   *   - extend enum Name Directives[Const]
   */
  ;

  _proto.parseEnumTypeExtension = function parseEnumTypeExtension() {
    var start = this._lexer.token;
    this.expectKeyword('extend');
    this.expectKeyword('enum');
    var name = this.parseName();
    var directives = this.parseDirectives(true);
    var values = this.parseEnumValuesDefinition();

    if (directives.length === 0 && values.length === 0) {
      throw this.unexpected();
    }

    return {
      kind: Kind.ENUM_TYPE_EXTENSION,
      name: name,
      directives: directives,
      values: values,
      loc: this.loc(start)
    };
  }
  /**
   * InputObjectTypeExtension :
   *   - extend input Name Directives[Const]? InputFieldsDefinition
   *   - extend input Name Directives[Const]
   */
  ;

  _proto.parseInputObjectTypeExtension = function parseInputObjectTypeExtension() {
    var start = this._lexer.token;
    this.expectKeyword('extend');
    this.expectKeyword('input');
    var name = this.parseName();
    var directives = this.parseDirectives(true);
    var fields = this.parseInputFieldsDefinition();

    if (directives.length === 0 && fields.length === 0) {
      throw this.unexpected();
    }

    return {
      kind: Kind.INPUT_OBJECT_TYPE_EXTENSION,
      name: name,
      directives: directives,
      fields: fields,
      loc: this.loc(start)
    };
  }
  /**
   * DirectiveDefinition :
   *   - Description? directive @ Name ArgumentsDefinition? `repeatable`? on DirectiveLocations
   */
  ;

  _proto.parseDirectiveDefinition = function parseDirectiveDefinition() {
    var start = this._lexer.token;
    var description = this.parseDescription();
    this.expectKeyword('directive');
    this.expectToken(TokenKind.AT);
    var name = this.parseName();
    var args = this.parseArgumentDefs();
    var repeatable = this.expectOptionalKeyword('repeatable');
    this.expectKeyword('on');
    var locations = this.parseDirectiveLocations();
    return {
      kind: Kind.DIRECTIVE_DEFINITION,
      description: description,
      name: name,
      arguments: args,
      repeatable: repeatable,
      locations: locations,
      loc: this.loc(start)
    };
  }
  /**
   * DirectiveLocations :
   *   - `|`? DirectiveLocation
   *   - DirectiveLocations | DirectiveLocation
   */
  ;

  _proto.parseDirectiveLocations = function parseDirectiveLocations() {
    return this.delimitedMany(TokenKind.PIPE, this.parseDirectiveLocation);
  }
  /*
   * DirectiveLocation :
   *   - ExecutableDirectiveLocation
   *   - TypeSystemDirectiveLocation
   *
   * ExecutableDirectiveLocation : one of
   *   `QUERY`
   *   `MUTATION`
   *   `SUBSCRIPTION`
   *   `FIELD`
   *   `FRAGMENT_DEFINITION`
   *   `FRAGMENT_SPREAD`
   *   `INLINE_FRAGMENT`
   *
   * TypeSystemDirectiveLocation : one of
   *   `SCHEMA`
   *   `SCALAR`
   *   `OBJECT`
   *   `FIELD_DEFINITION`
   *   `ARGUMENT_DEFINITION`
   *   `INTERFACE`
   *   `UNION`
   *   `ENUM`
   *   `ENUM_VALUE`
   *   `INPUT_OBJECT`
   *   `INPUT_FIELD_DEFINITION`
   */
  ;

  _proto.parseDirectiveLocation = function parseDirectiveLocation() {
    var start = this._lexer.token;
    var name = this.parseName();

    if (DirectiveLocation[name.value] !== undefined) {
      return name;
    }

    throw this.unexpected(start);
  } // Core parsing utility functions

  /**
   * Returns a location object, used to identify the place in the source that created a given parsed object.
   */
  ;

  _proto.loc = function loc(startToken) {
    var _this$_options4;

    if (((_this$_options4 = this._options) === null || _this$_options4 === void 0 ? void 0 : _this$_options4.noLocation) !== true) {
      return new Location(startToken, this._lexer.lastToken, this._lexer.source);
    }
  }
  /**
   * Determines if the next token is of a given kind
   */
  ;

  _proto.peek = function peek(kind) {
    return this._lexer.token.kind === kind;
  }
  /**
   * If the next token is of the given kind, return that token after advancing the lexer.
   * Otherwise, do not change the parser state and throw an error.
   */
  ;

  _proto.expectToken = function expectToken(kind) {
    var token = this._lexer.token;

    if (token.kind === kind) {
      this._lexer.advance();

      return token;
    }

    throw syntaxError(this._lexer.source, token.start, "Expected ".concat(getTokenKindDesc(kind), ", found ").concat(getTokenDesc(token), "."));
  }
  /**
   * If the next token is of the given kind, return that token after advancing the lexer.
   * Otherwise, do not change the parser state and return undefined.
   */
  ;

  _proto.expectOptionalToken = function expectOptionalToken(kind) {
    var token = this._lexer.token;

    if (token.kind === kind) {
      this._lexer.advance();

      return token;
    }

    return undefined;
  }
  /**
   * If the next token is a given keyword, advance the lexer.
   * Otherwise, do not change the parser state and throw an error.
   */
  ;

  _proto.expectKeyword = function expectKeyword(value) {
    var token = this._lexer.token;

    if (token.kind === TokenKind.NAME && token.value === value) {
      this._lexer.advance();
    } else {
      throw syntaxError(this._lexer.source, token.start, "Expected \"".concat(value, "\", found ").concat(getTokenDesc(token), "."));
    }
  }
  /**
   * If the next token is a given keyword, return "true" after advancing the lexer.
   * Otherwise, do not change the parser state and return "false".
   */
  ;

  _proto.expectOptionalKeyword = function expectOptionalKeyword(value) {
    var token = this._lexer.token;

    if (token.kind === TokenKind.NAME && token.value === value) {
      this._lexer.advance();

      return true;
    }

    return false;
  }
  /**
   * Helper function for creating an error when an unexpected lexed token is encountered.
   */
  ;

  _proto.unexpected = function unexpected(atToken) {
    var token = atToken !== null && atToken !== void 0 ? atToken : this._lexer.token;
    return syntaxError(this._lexer.source, token.start, "Unexpected ".concat(getTokenDesc(token), "."));
  }
  /**
   * Returns a possibly empty list of parse nodes, determined by the parseFn.
   * This list begins with a lex token of openKind and ends with a lex token of closeKind.
   * Advances the parser to the next lex token after the closing token.
   */
  ;

  _proto.any = function any(openKind, parseFn, closeKind) {
    this.expectToken(openKind);
    var nodes = [];

    while (!this.expectOptionalToken(closeKind)) {
      nodes.push(parseFn.call(this));
    }

    return nodes;
  }
  /**
   * Returns a list of parse nodes, determined by the parseFn.
   * It can be empty only if open token is missing otherwise it will always return non-empty list
   * that begins with a lex token of openKind and ends with a lex token of closeKind.
   * Advances the parser to the next lex token after the closing token.
   */
  ;

  _proto.optionalMany = function optionalMany(openKind, parseFn, closeKind) {
    if (this.expectOptionalToken(openKind)) {
      var nodes = [];

      do {
        nodes.push(parseFn.call(this));
      } while (!this.expectOptionalToken(closeKind));

      return nodes;
    }

    return [];
  }
  /**
   * Returns a non-empty list of parse nodes, determined by the parseFn.
   * This list begins with a lex token of openKind and ends with a lex token of closeKind.
   * Advances the parser to the next lex token after the closing token.
   */
  ;

  _proto.many = function many(openKind, parseFn, closeKind) {
    this.expectToken(openKind);
    var nodes = [];

    do {
      nodes.push(parseFn.call(this));
    } while (!this.expectOptionalToken(closeKind));

    return nodes;
  }
  /**
   * Returns a non-empty list of parse nodes, determined by the parseFn.
   * This list may begin with a lex token of delimiterKind followed by items separated by lex tokens of tokenKind.
   * Advances the parser to the next lex token after last item in the list.
   */
  ;

  _proto.delimitedMany = function delimitedMany(delimiterKind, parseFn) {
    this.expectOptionalToken(delimiterKind);
    var nodes = [];

    do {
      nodes.push(parseFn.call(this));
    } while (this.expectOptionalToken(delimiterKind));

    return nodes;
  };

  return Parser;
}();
/**
 * A helper function to describe a token as a string for debugging.
 */

function getTokenDesc(token) {
  var value = token.value;
  return getTokenKindDesc(token.kind) + (value != null ? " \"".concat(value, "\"") : '');
}
/**
 * A helper function to describe a token kind as a string for debugging.
 */


function getTokenKindDesc(kind) {
  return isPunctuatorTokenKind(kind) ? "\"".concat(kind, "\"") : kind;
}

var docCache = new Map();
var fragmentSourceMap = new Map();
var printFragmentWarnings = true;
var experimentalFragmentVariables = false;
function normalize(string) {
    return string.replace(/[\s,]+/g, ' ').trim();
}
function cacheKeyFromLoc(loc) {
    return normalize(loc.source.body.substring(loc.start, loc.end));
}
function processFragments(ast) {
    var seenKeys = new Set();
    var definitions = [];
    ast.definitions.forEach(function (fragmentDefinition) {
        if (fragmentDefinition.kind === 'FragmentDefinition') {
            var fragmentName = fragmentDefinition.name.value;
            var sourceKey = cacheKeyFromLoc(fragmentDefinition.loc);
            var sourceKeySet = fragmentSourceMap.get(fragmentName);
            if (sourceKeySet && !sourceKeySet.has(sourceKey)) {
                if (printFragmentWarnings) {
                    console.warn("Warning: fragment with name " + fragmentName + " already exists.\n"
                        + "graphql-tag enforces all fragment names across your application to be unique; read more about\n"
                        + "this in the docs: http://dev.apollodata.com/core/fragments.html#unique-names");
                }
            }
            else if (!sourceKeySet) {
                fragmentSourceMap.set(fragmentName, sourceKeySet = new Set);
            }
            sourceKeySet.add(sourceKey);
            if (!seenKeys.has(sourceKey)) {
                seenKeys.add(sourceKey);
                definitions.push(fragmentDefinition);
            }
        }
        else {
            definitions.push(fragmentDefinition);
        }
    });
    return __assign(__assign({}, ast), { definitions: definitions });
}
function stripLoc(doc) {
    var workSet = new Set(doc.definitions);
    workSet.forEach(function (node) {
        if (node.loc)
            delete node.loc;
        Object.keys(node).forEach(function (key) {
            var value = node[key];
            if (value && typeof value === 'object') {
                workSet.add(value);
            }
        });
    });
    var loc = doc.loc;
    if (loc) {
        delete loc.startToken;
        delete loc.endToken;
    }
    return doc;
}
function parseDocument(source) {
    var cacheKey = normalize(source);
    if (!docCache.has(cacheKey)) {
        var parsed = parse(source, {
            experimentalFragmentVariables: experimentalFragmentVariables,
            allowLegacyFragmentVariables: experimentalFragmentVariables
        });
        if (!parsed || parsed.kind !== 'Document') {
            throw new Error('Not a valid GraphQL document.');
        }
        docCache.set(cacheKey, stripLoc(processFragments(parsed)));
    }
    return docCache.get(cacheKey);
}
function gql(literals) {
    var args = [];
    for (var _i = 1; _i < arguments.length; _i++) {
        args[_i - 1] = arguments[_i];
    }
    if (typeof literals === 'string') {
        literals = [literals];
    }
    var result = literals[0];
    args.forEach(function (arg, i) {
        if (arg && arg.kind === 'Document') {
            result += arg.loc.source.body;
        }
        else {
            result += arg;
        }
        result += literals[i + 1];
    });
    return parseDocument(result);
}
function resetCaches() {
    docCache.clear();
    fragmentSourceMap.clear();
}
function disableFragmentWarnings() {
    printFragmentWarnings = false;
}
function enableExperimentalFragmentVariables() {
    experimentalFragmentVariables = true;
}
function disableExperimentalFragmentVariables() {
    experimentalFragmentVariables = false;
}
var extras = {
    gql: gql,
    resetCaches: resetCaches,
    disableFragmentWarnings: disableFragmentWarnings,
    enableExperimentalFragmentVariables: enableExperimentalFragmentVariables,
    disableExperimentalFragmentVariables: disableExperimentalFragmentVariables
};
(function (gql_1) {
    gql_1.gql = extras.gql, gql_1.resetCaches = extras.resetCaches, gql_1.disableFragmentWarnings = extras.disableFragmentWarnings, gql_1.enableExperimentalFragmentVariables = extras.enableExperimentalFragmentVariables, gql_1.disableExperimentalFragmentVariables = extras.disableExperimentalFragmentVariables;
})(gql || (gql = {}));
gql["default"] = gql;

function unwrapApolloResult(result) {
    //console.debug('GQL result:', result)
    //@ts-ignore
    if (result.error) {
        //@ts-ignore
        throw result.error;
    }
    if (result.errors) {
        throw result.errors;
    }
    return result.data;
}

/*
  This "shim" can be used on the frontend to prevent from errors on undefined decorators,
  when you are sharing same classes across backend and frontend.

  To use this shim, simply set up your Webpack configuration
  to use this file instead of a normal TypeGraphQL module.

  ```js
  plugins: [
    // ...here are any other existing plugins that you already have
    new webpack.NormalModuleReplacementPlugin(/type-graphql$/, resource => {
      resource.request = resource.request.replace(/type-graphql/, "type-graphql/dist/browser-shim");
    }),
  ]
  ```

  However, in some TypeScript projects like the ones using Angular,
  which AoT compiler requires that a full `*.ts` file is provided
  instead of just a `*.js` and `*.d.ts` files, to use this shim
  we have to simply set up our TypeScript configuration in `tsconfig.json`
  to use this file instead of a normal TypeGraphQL module:

  ```json
  {
    "compilerOptions": {
      "baseUrl": ".",
      "paths": {
        "type-graphql": [
          "./node_modules/type-graphql/dist/browser-shim.ts"
        ]
      }
    }
  }
  ```
*/
function dummyFn() {
    return;
}
function dummyDecorator() {
    return dummyFn;
}
const Field = dummyDecorator;
const InputType = dummyDecorator;
const ObjectType = dummyDecorator;

var __decorate$e = function (decorators, target, key, desc) {
    var c = arguments.length, r = c < 3 ? target : desc === null ? desc = Object.getOwnPropertyDescriptor(target, key) : desc, d;
    if (typeof Reflect === "object" && typeof Reflect.decorate === "function") r = Reflect.decorate(decorators, target, key, desc);
    else for (var i = decorators.length - 1; i >= 0; i--) if (d = decorators[i]) r = (c < 3 ? d(r) : c > 3 ? d(target, key, r) : d(target, key)) || r;
    return c > 3 && r && Object.defineProperty(target, key, r), r;
};
var __metadata$e = function (k, v) {
    if (typeof Reflect === "object" && typeof Reflect.metadata === "function") return Reflect.metadata(k, v);
};
let Icon = class Icon {
    constructor(code) {
        this.code = code;
    }
};
__decorate$e([
    Field(),
    __metadata$e("design:type", String)
], Icon.prototype, "code", void 0);
Icon = __decorate$e([
    ObjectType(),
    __metadata$e("design:paramtypes", [String])
], Icon);

var __decorate$d = function (decorators, target, key, desc) {
    var c = arguments.length, r = c < 3 ? target : desc === null ? desc = Object.getOwnPropertyDescriptor(target, key) : desc, d;
    if (typeof Reflect === "object" && typeof Reflect.decorate === "function") r = Reflect.decorate(decorators, target, key, desc);
    else for (var i = decorators.length - 1; i >= 0; i--) if (d = decorators[i]) r = (c < 3 ? d(r) : c > 3 ? d(target, key, r) : d(target, key)) || r;
    return c > 3 && r && Object.defineProperty(target, key, r), r;
};
var __metadata$d = function (k, v) {
    if (typeof Reflect === "object" && typeof Reflect.metadata === "function") return Reflect.metadata(k, v);
};
// Unique Language ID with option type
let LanguageRef = class LanguageRef {
    constructor(address, name) {
        this.address = address;
        this.name = name;
    }
};
__decorate$d([
    Field(),
    __metadata$d("design:type", String)
], LanguageRef.prototype, "address", void 0);
__decorate$d([
    Field(),
    __metadata$d("design:type", String)
], LanguageRef.prototype, "name", void 0);
LanguageRef = __decorate$d([
    ObjectType(),
    __metadata$d("design:paramtypes", [String, String])
], LanguageRef);

var __decorate$c = function (decorators, target, key, desc) {
    var c = arguments.length, r = c < 3 ? target : desc === null ? desc = Object.getOwnPropertyDescriptor(target, key) : desc, d;
    if (typeof Reflect === "object" && typeof Reflect.decorate === "function") r = Reflect.decorate(decorators, target, key, desc);
    else for (var i = decorators.length - 1; i >= 0; i--) if (d = decorators[i]) r = (c < 3 ? d(r) : c > 3 ? d(target, key, r) : d(target, key)) || r;
    return c > 3 && r && Object.defineProperty(target, key, r), r;
};
var __metadata$c = function (k, v) {
    if (typeof Reflect === "object" && typeof Reflect.metadata === "function") return Reflect.metadata(k, v);
};
let ExpressionProof = class ExpressionProof {
    constructor(sig, k) {
        this.key = k;
        this.signature = sig;
    }
};
__decorate$c([
    Field(),
    __metadata$c("design:type", String)
], ExpressionProof.prototype, "signature", void 0);
__decorate$c([
    Field(),
    __metadata$c("design:type", String)
], ExpressionProof.prototype, "key", void 0);
__decorate$c([
    Field(),
    __metadata$c("design:type", Boolean)
], ExpressionProof.prototype, "valid", void 0);
__decorate$c([
    Field(),
    __metadata$c("design:type", Boolean)
], ExpressionProof.prototype, "invalid", void 0);
ExpressionProof = __decorate$c([
    ObjectType(),
    InputType(),
    __metadata$c("design:paramtypes", [String, String])
], ExpressionProof);
let ExpressionProofInput = class ExpressionProofInput {
};
__decorate$c([
    Field(),
    __metadata$c("design:type", String)
], ExpressionProofInput.prototype, "signature", void 0);
__decorate$c([
    Field(),
    __metadata$c("design:type", String)
], ExpressionProofInput.prototype, "key", void 0);
__decorate$c([
    Field(),
    __metadata$c("design:type", Boolean)
], ExpressionProofInput.prototype, "valid", void 0);
__decorate$c([
    Field(),
    __metadata$c("design:type", Boolean)
], ExpressionProofInput.prototype, "invalid", void 0);
ExpressionProofInput = __decorate$c([
    InputType()
], ExpressionProofInput);
//Note having any as return type here fixes compilation errors but I think we loose the ExpressionClass type in resulting .d.ts gql files
function ExpressionGeneric(DataTypeClass) {
    let ExpressionClass = class ExpressionClass {
        constructor(author, timestamp, data, proof) {
            this.author = author;
            this.timestamp = timestamp;
            this.data = data;
            this.proof = proof;
        }
    };
    __decorate$c([
        Field(),
        __metadata$c("design:type", String)
    ], ExpressionClass.prototype, "author", void 0);
    __decorate$c([
        Field(),
        __metadata$c("design:type", String)
    ], ExpressionClass.prototype, "timestamp", void 0);
    __decorate$c([
        Field(),
        __metadata$c("design:type", Object)
    ], ExpressionClass.prototype, "data", void 0);
    __decorate$c([
        Field(),
        __metadata$c("design:type", ExpressionProof)
    ], ExpressionClass.prototype, "proof", void 0);
    ExpressionClass = __decorate$c([
        ObjectType(),
        __metadata$c("design:paramtypes", [String, String, Object, ExpressionProof])
    ], ExpressionClass);
    return ExpressionClass;
}
function ExpressionGenericInput(DataTypeClass) {
    let ExpressionClass = class ExpressionClass {
    };
    __decorate$c([
        Field(),
        __metadata$c("design:type", String)
    ], ExpressionClass.prototype, "author", void 0);
    __decorate$c([
        Field(),
        __metadata$c("design:type", String)
    ], ExpressionClass.prototype, "timestamp", void 0);
    __decorate$c([
        Field(),
        __metadata$c("design:type", Object)
    ], ExpressionClass.prototype, "data", void 0);
    __decorate$c([
        Field(),
        __metadata$c("design:type", ExpressionProofInput)
    ], ExpressionClass.prototype, "proof", void 0);
    ExpressionClass = __decorate$c([
        InputType()
    ], ExpressionClass);
    return ExpressionClass;
}
let Expression = class Expression extends ExpressionGeneric() {
};
Expression = __decorate$c([
    ObjectType()
], Expression);
let ExpressionRendered = class ExpressionRendered extends ExpressionGeneric() {
};
__decorate$c([
    Field(),
    __metadata$c("design:type", LanguageRef)
], ExpressionRendered.prototype, "language", void 0);
__decorate$c([
    Field(),
    __metadata$c("design:type", Icon)
], ExpressionRendered.prototype, "icon", void 0);
ExpressionRendered = __decorate$c([
    ObjectType()
], ExpressionRendered);

var __decorate$b = function (decorators, target, key, desc) {
    var c = arguments.length, r = c < 3 ? target : desc === null ? desc = Object.getOwnPropertyDescriptor(target, key) : desc, d;
    if (typeof Reflect === "object" && typeof Reflect.decorate === "function") r = Reflect.decorate(decorators, target, key, desc);
    else for (var i = decorators.length - 1; i >= 0; i--) if (d = decorators[i]) r = (c < 3 ? d(r) : c > 3 ? d(target, key, r) : d(target, key)) || r;
    return c > 3 && r && Object.defineProperty(target, key, r), r;
};
var __metadata$b = function (k, v) {
    if (typeof Reflect === "object" && typeof Reflect.metadata === "function") return Reflect.metadata(k, v);
};
let Link = class Link {
    constructor(obj) {
        this.source = obj.source ? obj.source : '';
        this.target = obj.target ? obj.target : '';
        this.predicate = obj.predicate ? obj.predicate : '';
    }
};
__decorate$b([
    Field(),
    __metadata$b("design:type", String)
], Link.prototype, "source", void 0);
__decorate$b([
    Field(),
    __metadata$b("design:type", String)
], Link.prototype, "target", void 0);
__decorate$b([
    Field(),
    __metadata$b("design:type", String)
], Link.prototype, "predicate", void 0);
Link = __decorate$b([
    ObjectType(),
    __metadata$b("design:paramtypes", [Object])
], Link);
let LinkMutations = class LinkMutations {
};
__decorate$b([
    Field(),
    __metadata$b("design:type", Array)
], LinkMutations.prototype, "additions", void 0);
__decorate$b([
    Field(),
    __metadata$b("design:type", Array)
], LinkMutations.prototype, "removals", void 0);
LinkMutations = __decorate$b([
    InputType()
], LinkMutations);
let LinkExpressionMutations = class LinkExpressionMutations {
    constructor(additions, removals) {
        this.additions = additions;
        this.removals = removals;
    }
};
__decorate$b([
    Field(),
    __metadata$b("design:type", Array)
], LinkExpressionMutations.prototype, "additions", void 0);
__decorate$b([
    Field(),
    __metadata$b("design:type", Array)
], LinkExpressionMutations.prototype, "removals", void 0);
LinkExpressionMutations = __decorate$b([
    ObjectType(),
    __metadata$b("design:paramtypes", [Array, Array])
], LinkExpressionMutations);
let LinkInput = class LinkInput {
};
__decorate$b([
    Field(),
    __metadata$b("design:type", String)
], LinkInput.prototype, "source", void 0);
__decorate$b([
    Field(),
    __metadata$b("design:type", String)
], LinkInput.prototype, "target", void 0);
__decorate$b([
    Field(),
    __metadata$b("design:type", String)
], LinkInput.prototype, "predicate", void 0);
LinkInput = __decorate$b([
    InputType()
], LinkInput);
let LinkExpression = class LinkExpression extends ExpressionGeneric() {
    hash() {
        const mash = JSON.stringify(this.data, Object.keys(this.data).sort()) +
            JSON.stringify(this.author) + this.timestamp;
        let hash = 0, i, chr;
        for (i = 0; i < mash.length; i++) {
            chr = mash.charCodeAt(i);
            hash = ((hash << 5) - hash) + chr;
            hash |= 0; // Convert to 32bit integer
        }
        return hash;
    }
};
__decorate$b([
    Field(),
    __metadata$b("design:type", String)
], LinkExpression.prototype, "status", void 0);
LinkExpression = __decorate$b([
    ObjectType()
], LinkExpression);
let LinkExpressionInput = class LinkExpressionInput extends ExpressionGenericInput() {
};
__decorate$b([
    Field(),
    __metadata$b("design:type", String)
], LinkExpressionInput.prototype, "status", void 0);
LinkExpressionInput = __decorate$b([
    InputType()
], LinkExpressionInput);
let LinkExpressionUpdated = class LinkExpressionUpdated {
    constructor(oldLink, newLink) {
        this.oldLink = oldLink;
        this.newLink = newLink;
    }
};
__decorate$b([
    Field(),
    __metadata$b("design:type", LinkExpression)
], LinkExpressionUpdated.prototype, "oldLink", void 0);
__decorate$b([
    Field(),
    __metadata$b("design:type", LinkExpression)
], LinkExpressionUpdated.prototype, "newLink", void 0);
LinkExpressionUpdated = __decorate$b([
    ObjectType(),
    __metadata$b("design:paramtypes", [LinkExpression, LinkExpression])
], LinkExpressionUpdated);

var __decorate$a = function (decorators, target, key, desc) {
    var c = arguments.length, r = c < 3 ? target : desc === null ? desc = Object.getOwnPropertyDescriptor(target, key) : desc, d;
    if (typeof Reflect === "object" && typeof Reflect.decorate === "function") r = Reflect.decorate(decorators, target, key, desc);
    else for (var i = decorators.length - 1; i >= 0; i--) if (d = decorators[i]) r = (c < 3 ? d(r) : c > 3 ? d(target, key, r) : d(target, key)) || r;
    return c > 3 && r && Object.defineProperty(target, key, r), r;
};
var __metadata$a = function (k, v) {
    if (typeof Reflect === "object" && typeof Reflect.metadata === "function") return Reflect.metadata(k, v);
};
var PerspectiveUnsignedInput_1;
/** A Perspective represents subjective meaning, encoded through
* associations between expressions, a.k.a. Links, that is a graph
* over the objective Expressions of any subset of Languages.
*
* This type represents the clean onotological concept of a Perspective.
* An instance of this class can be regarded as an immutable snapshot of
* a mutable perspective.
*
* The types PerspectiveProxy and PerspectiveHandle are used when dealing
* with an instantiated mutable perspective as is done through most of
* the GraphQL mutations.
*/
let Perspective = class Perspective {
    constructor(links) {
        if (links) {
            this.links = links;
        }
        else {
            this.links = [];
        }
    }
    /** Convenience function for filtering links just like with PerspectiveProxy */
    get(query) {
        if (!query || !query.source && !query.predicate && !query.target) {
            return this.links;
        }
        if (query.source) {
            let result = JSON.parse(JSON.stringify(this.links));
            // @ts-ignore
            if (query.target)
                result = result.filter(l => l.data.target === query.target);
            // @ts-ignore
            if (query.predicate)
                result = result.filter(l => l.data.predicate === query.predicate);
            //@ts-ignore
            if (query.fromDate)
                result = result.filter(l => new Date(l.timestamp) >= query.fromDate);
            //@ts-ignore
            if (query.untilDate)
                result = result.filter(l => new Date(l.timestamp) <= query.untilDate);
            // console.debug("result", result)
            if (query.limit)
                result = result.slice(0, query.limit);
            return result;
        }
        // console.debug("getLinks 3")
        if (query.target) {
            //@ts-ignore
            let result = JSON.parse(JSON.stringify(this.links));
            // @ts-ignore
            if (query.predicate)
                result = result.filter(l => l.data.predicate === query.predicate);
            //@ts-ignore
            if (query.fromDate)
                result = result.filter(l => new Date(l.timestamp) >= query.fromDate);
            //@ts-ignore
            if (query.untilDate)
                result = result.filter(l => new Date(l.timestamp) <= query.untilDate);
            if (query.limit)
                result = result.slice(0, query.limit);
            return result;
        }
        // console.debug("getLinks 4")
        //@ts-ignore
        let result = JSON.parse(JSON.stringify(this.links));
        result = result.filter(link => link.data.predicate === query.predicate);
        if (query.limit)
            result = result.slice(0, query.limit);
        return result;
    }
    /** Convenience function to get the target of the first link that matches the given query
     * This makes sense when the query is expected to return only one link
     * and the target of that link is what you are looking for.
     */
    getSingleTarget(query) {
        delete query.target;
        const foundLinks = this.get(query);
        if (foundLinks.length)
            return foundLinks[0].data.target;
        else
            return null;
    }
};
__decorate$a([
    Field(),
    __metadata$a("design:type", Array)
], Perspective.prototype, "links", void 0);
Perspective = __decorate$a([
    ObjectType(),
    __metadata$a("design:paramtypes", [Array])
], Perspective);
let PerspectiveInput = class PerspectiveInput {
};
__decorate$a([
    Field(),
    __metadata$a("design:type", Array)
], PerspectiveInput.prototype, "links", void 0);
PerspectiveInput = __decorate$a([
    InputType()
], PerspectiveInput);
let PerspectiveUnsignedInput = PerspectiveUnsignedInput_1 = class PerspectiveUnsignedInput {
    constructor(links) {
        if (links)
            this.links = links;
        else
            this.links = [];
    }
    static fromLink(link) {
        let obj = new PerspectiveUnsignedInput_1();
        obj.links.push(link);
        return obj;
    }
};
__decorate$a([
    Field(),
    __metadata$a("design:type", Array)
], PerspectiveUnsignedInput.prototype, "links", void 0);
PerspectiveUnsignedInput = PerspectiveUnsignedInput_1 = __decorate$a([
    InputType(),
    __metadata$a("design:paramtypes", [Array])
], PerspectiveUnsignedInput);
let PerspectiveExpression = class PerspectiveExpression extends ExpressionGeneric() {
};
PerspectiveExpression = __decorate$a([
    ObjectType()
], PerspectiveExpression);

var __decorate$9 = function (decorators, target, key, desc) {
    var c = arguments.length, r = c < 3 ? target : desc === null ? desc = Object.getOwnPropertyDescriptor(target, key) : desc, d;
    if (typeof Reflect === "object" && typeof Reflect.decorate === "function") r = Reflect.decorate(decorators, target, key, desc);
    else for (var i = decorators.length - 1; i >= 0; i--) if (d = decorators[i]) r = (c < 3 ? d(r) : c > 3 ? d(target, key, r) : d(target, key)) || r;
    return c > 3 && r && Object.defineProperty(target, key, r), r;
};
var __metadata$9 = function (k, v) {
    if (typeof Reflect === "object" && typeof Reflect.metadata === "function") return Reflect.metadata(k, v);
};
/**  AD4M's representation of an Agent
 *
 * AD4M Agents are build around DIDs, which are used to identify and authenticate the Agent.
 * Conceptually, an Agent is regarded as something that can speak and that can listen.
 *
 * Agents speak by creating Expressions in AD4M Languages which are signed by the Agent's DID key,
 * And they also speak (broadcast) by putting semantic statements into their public "Agent Perspective".
 * They listen (can receive messages) through their "direct message Language".
 *
 * These three aspects are represented by the three fields of this class.
 *
 * This class is used as format for the Expressions in the Agent language.
 * Since AD4M treats DID URIs as addresses for the Agent Language,
 * DIDs are resolved to Expressions that are objects of this class.
 * Thus, this is how agents see (other) agents.
 */
let Agent = class Agent {
    constructor(did, perspective) {
        this.did = did;
        if (perspective) {
            this.perspective = perspective;
        }
        else {
            this.perspective = new Perspective();
        }
    }
};
__decorate$9([
    Field(),
    __metadata$9("design:type", String)
], Agent.prototype, "did", void 0);
__decorate$9([
    Field(),
    __metadata$9("design:type", Perspective)
], Agent.prototype, "perspective", void 0);
__decorate$9([
    Field(),
    __metadata$9("design:type", String)
], Agent.prototype, "directMessageLanguage", void 0);
Agent = __decorate$9([
    ObjectType(),
    __metadata$9("design:paramtypes", [String, Perspective])
], Agent);
let AgentExpression = class AgentExpression extends ExpressionGeneric() {
};
AgentExpression = __decorate$9([
    ObjectType()
], AgentExpression);
let EntanglementProof = class EntanglementProof {
    constructor(did, didSigningKeyId, deviceKeyType, deviceKey, deviceKeySignedByDid, didSignedByDeviceKey) {
        this.did = did;
        this.didSigningKeyId = didSigningKeyId;
        this.deviceKeyType = deviceKeyType;
        this.deviceKey = deviceKey;
        this.deviceKeySignedByDid = deviceKeySignedByDid;
        this.didSignedByDeviceKey = didSignedByDeviceKey;
    }
};
__decorate$9([
    Field(),
    __metadata$9("design:type", String)
], EntanglementProof.prototype, "did", void 0);
__decorate$9([
    Field(),
    __metadata$9("design:type", String)
], EntanglementProof.prototype, "didSigningKeyId", void 0);
__decorate$9([
    Field(),
    __metadata$9("design:type", String)
], EntanglementProof.prototype, "deviceKeyType", void 0);
__decorate$9([
    Field(),
    __metadata$9("design:type", String)
], EntanglementProof.prototype, "deviceKey", void 0);
__decorate$9([
    Field(),
    __metadata$9("design:type", String)
], EntanglementProof.prototype, "deviceKeySignedByDid", void 0);
__decorate$9([
    Field(),
    __metadata$9("design:type", String)
], EntanglementProof.prototype, "didSignedByDeviceKey", void 0);
EntanglementProof = __decorate$9([
    ObjectType(),
    __metadata$9("design:paramtypes", [String, String, String, String, String, String])
], EntanglementProof);
let EntanglementProofInput = class EntanglementProofInput {
    constructor(did, didSigningKeyId, deviceKeyType, deviceKey, deviceKeySignedByDid, didSignedByDeviceKey) {
        this.did = did;
        this.didSigningKeyId = didSigningKeyId;
        this.deviceKeyType = deviceKeyType;
        this.deviceKey = deviceKey;
        this.deviceKeySignedByDid = deviceKeySignedByDid;
        this.didSignedByDeviceKey = didSignedByDeviceKey;
    }
};
__decorate$9([
    Field(),
    __metadata$9("design:type", String)
], EntanglementProofInput.prototype, "did", void 0);
__decorate$9([
    Field(),
    __metadata$9("design:type", String)
], EntanglementProofInput.prototype, "didSigningKeyId", void 0);
__decorate$9([
    Field(),
    __metadata$9("design:type", String)
], EntanglementProofInput.prototype, "deviceKeyType", void 0);
__decorate$9([
    Field(),
    __metadata$9("design:type", String)
], EntanglementProofInput.prototype, "deviceKey", void 0);
__decorate$9([
    Field(),
    __metadata$9("design:type", String)
], EntanglementProofInput.prototype, "deviceKeySignedByDid", void 0);
__decorate$9([
    Field(),
    __metadata$9("design:type", String)
], EntanglementProofInput.prototype, "didSignedByDeviceKey", void 0);
EntanglementProofInput = __decorate$9([
    InputType(),
    __metadata$9("design:paramtypes", [String, String, String, String, String, String])
], EntanglementProofInput);
let AgentSignature = class AgentSignature {
    constructor(signature, publicKey) {
        this.signature = signature;
        this.publicKey = publicKey;
    }
};
__decorate$9([
    Field(),
    __metadata$9("design:type", String)
], AgentSignature.prototype, "signature", void 0);
__decorate$9([
    Field(),
    __metadata$9("design:type", String)
], AgentSignature.prototype, "publicKey", void 0);
AgentSignature = __decorate$9([
    ObjectType(),
    __metadata$9("design:paramtypes", [String, String])
], AgentSignature);
let Resource = class Resource {
    constructor(domain, pointers) {
        this.domain = domain;
        this.pointers = pointers;
    }
};
__decorate$9([
    Field(),
    __metadata$9("design:type", String)
], Resource.prototype, "domain", void 0);
__decorate$9([
    Field(),
    __metadata$9("design:type", Array)
], Resource.prototype, "pointers", void 0);
Resource = __decorate$9([
    ObjectType(),
    __metadata$9("design:paramtypes", [String, Array])
], Resource);
let Capability = class Capability {
    constructor(withF, can) {
        this.with = withF;
        this.can = can;
    }
};
__decorate$9([
    Field(),
    __metadata$9("design:type", Resource)
], Capability.prototype, "with", void 0);
__decorate$9([
    Field(),
    __metadata$9("design:type", Array)
], Capability.prototype, "can", void 0);
Capability = __decorate$9([
    ObjectType(),
    __metadata$9("design:paramtypes", [Resource, Array])
], Capability);
let AuthInfo = class AuthInfo {
    constructor(appName, appDesc, appUrl, capabilities, appIconPath) {
        this.appName = appName;
        this.appDesc = appDesc;
        this.appIconPath = appIconPath;
        this.appUrl = appUrl;
        this.capabilities = capabilities;
    }
};
__decorate$9([
    Field(),
    __metadata$9("design:type", String)
], AuthInfo.prototype, "appName", void 0);
__decorate$9([
    Field(),
    __metadata$9("design:type", String)
], AuthInfo.prototype, "appDesc", void 0);
__decorate$9([
    Field(),
    __metadata$9("design:type", String)
], AuthInfo.prototype, "appUrl", void 0);
__decorate$9([
    Field(),
    __metadata$9("design:type", String)
], AuthInfo.prototype, "appIconPath", void 0);
__decorate$9([
    Field(),
    __metadata$9("design:type", Array)
], AuthInfo.prototype, "capabilities", void 0);
AuthInfo = __decorate$9([
    ObjectType(),
    __metadata$9("design:paramtypes", [String, String, String, Array, String])
], AuthInfo);
let Apps = class Apps {
    constructor(requestId, auth, token, revoked) {
        this.requestId = requestId;
        this.auth = auth;
        this.token = token;
        this.revoked = revoked;
    }
};
__decorate$9([
    Field(),
    __metadata$9("design:type", String)
], Apps.prototype, "requestId", void 0);
__decorate$9([
    Field(),
    __metadata$9("design:type", String)
], Apps.prototype, "token", void 0);
__decorate$9([
    Field(),
    __metadata$9("design:type", Boolean)
], Apps.prototype, "revoked", void 0);
__decorate$9([
    Field(),
    __metadata$9("design:type", AuthInfo)
], Apps.prototype, "auth", void 0);
Apps = __decorate$9([
    ObjectType(),
    __metadata$9("design:paramtypes", [String, AuthInfo, String, Boolean])
], Apps);
let ResourceInput = class ResourceInput {
    constructor(domain, pointers) {
        this.domain = domain;
        this.pointers = pointers;
    }
};
__decorate$9([
    Field(),
    __metadata$9("design:type", String)
], ResourceInput.prototype, "domain", void 0);
__decorate$9([
    Field(),
    __metadata$9("design:type", Array)
], ResourceInput.prototype, "pointers", void 0);
ResourceInput = __decorate$9([
    InputType(),
    __metadata$9("design:paramtypes", [String, Array])
], ResourceInput);
let CapabilityInput = class CapabilityInput {
    constructor(withF, can) {
        this.with = withF;
        this.can = can;
    }
};
__decorate$9([
    Field(),
    __metadata$9("design:type", ResourceInput)
], CapabilityInput.prototype, "with", void 0);
__decorate$9([
    Field(),
    __metadata$9("design:type", Array)
], CapabilityInput.prototype, "can", void 0);
CapabilityInput = __decorate$9([
    InputType(),
    __metadata$9("design:paramtypes", [ResourceInput, Array])
], CapabilityInput);
let AuthInfoInput = class AuthInfoInput {
    constructor(appName, appDesc, appDomain, appUrl, appIconPath, capabilities) {
        this.appName = appName;
        this.appDesc = appDesc;
        this.appUrl = appUrl;
        this.appDomain = appDomain;
        this.capabilities = capabilities;
        this.appIconPath = appIconPath;
    }
};
__decorate$9([
    Field(),
    __metadata$9("design:type", String)
], AuthInfoInput.prototype, "appName", void 0);
__decorate$9([
    Field(),
    __metadata$9("design:type", String)
], AuthInfoInput.prototype, "appDesc", void 0);
__decorate$9([
    Field(),
    __metadata$9("design:type", String)
], AuthInfoInput.prototype, "appDomain", void 0);
__decorate$9([
    Field(),
    __metadata$9("design:type", String)
], AuthInfoInput.prototype, "appUrl", void 0);
__decorate$9([
    Field(),
    __metadata$9("design:type", String)
], AuthInfoInput.prototype, "appIconPath", void 0);
__decorate$9([
    Field(),
    __metadata$9("design:type", Array)
], AuthInfoInput.prototype, "capabilities", void 0);
AuthInfoInput = __decorate$9([
    InputType(),
    __metadata$9("design:paramtypes", [String, String, String, String, String, Array])
], AuthInfoInput);

var __decorate$8 = function (decorators, target, key, desc) {
    var c = arguments.length, r = c < 3 ? target : desc === null ? desc = Object.getOwnPropertyDescriptor(target, key) : desc, d;
    if (typeof Reflect === "object" && typeof Reflect.decorate === "function") r = Reflect.decorate(decorators, target, key, desc);
    else for (var i = decorators.length - 1; i >= 0; i--) if (d = decorators[i]) r = (c < 3 ? d(r) : c > 3 ? d(target, key, r) : d(target, key)) || r;
    return c > 3 && r && Object.defineProperty(target, key, r), r;
};
var __metadata$8 = function (k, v) {
    if (typeof Reflect === "object" && typeof Reflect.metadata === "function") return Reflect.metadata(k, v);
};
let AgentStatus = class AgentStatus {
    constructor(obj) {
        if (obj) {
            //@ts-ignore
            this.isInitialized = obj.isInitialized;
            if (!this.isInitialized) {
                this.isInitialized = false;
            }
            //@ts-ignore
            this.isUnlocked = obj.isUnlocked;
            if (!this.isUnlocked) {
                this.isUnlocked = false;
            }
            //@ts-ignore
            this.did = obj.did;
            //@ts-ignore
            this.didDocument = obj.didDocument;
            //@ts-ignore
            this.error = obj.error;
        }
        else {
            this.isInitialized = false;
            this.isUnlocked = false;
        }
    }
};
__decorate$8([
    Field(),
    __metadata$8("design:type", Boolean)
], AgentStatus.prototype, "isInitialized", void 0);
__decorate$8([
    Field(),
    __metadata$8("design:type", Boolean)
], AgentStatus.prototype, "isUnlocked", void 0);
__decorate$8([
    Field(),
    __metadata$8("design:type", String)
], AgentStatus.prototype, "did", void 0);
__decorate$8([
    Field(),
    __metadata$8("design:type", String)
], AgentStatus.prototype, "didDocument", void 0);
__decorate$8([
    Field(),
    __metadata$8("design:type", String)
], AgentStatus.prototype, "error", void 0);
AgentStatus = __decorate$8([
    ObjectType(),
    __metadata$8("design:paramtypes", [Object])
], AgentStatus);

var __classPrivateFieldSet$c = function (receiver, state, value, kind, f) {
    if (kind === "m") throw new TypeError("Private method is not writable");
    if (kind === "a" && !f) throw new TypeError("Private accessor was defined without a setter");
    if (typeof state === "function" ? receiver !== state || !f : !state.has(receiver)) throw new TypeError("Cannot write private member to an object whose class did not declare it");
    return (kind === "a" ? f.call(receiver, value) : f ? f.value = value : state.set(receiver, value)), value;
};
var __classPrivateFieldGet$c = function (receiver, state, kind, f) {
    if (kind === "a" && !f) throw new TypeError("Private accessor was defined without a getter");
    if (typeof state === "function" ? receiver !== state || !f : !state.has(receiver)) throw new TypeError("Cannot read private member from an object whose class did not declare it");
    return kind === "m" ? f : kind === "a" ? f.call(receiver) : f ? f.value : state.get(receiver);
};
var _NeighbourhoodProxy_client, _NeighbourhoodProxy_pID;
class NeighbourhoodProxy {
    constructor(client, pID) {
        _NeighbourhoodProxy_client.set(this, void 0);
        _NeighbourhoodProxy_pID.set(this, void 0);
        __classPrivateFieldSet$c(this, _NeighbourhoodProxy_client, client, "f");
        __classPrivateFieldSet$c(this, _NeighbourhoodProxy_pID, pID, "f");
    }
    async otherAgents() {
        return await __classPrivateFieldGet$c(this, _NeighbourhoodProxy_client, "f").otherAgents(__classPrivateFieldGet$c(this, _NeighbourhoodProxy_pID, "f"));
    }
    async hasTelepresenceAdapter() {
        return await __classPrivateFieldGet$c(this, _NeighbourhoodProxy_client, "f").hasTelepresenceAdapter(__classPrivateFieldGet$c(this, _NeighbourhoodProxy_pID, "f"));
    }
    async onlineAgents() {
        return await __classPrivateFieldGet$c(this, _NeighbourhoodProxy_client, "f").onlineAgents(__classPrivateFieldGet$c(this, _NeighbourhoodProxy_pID, "f"));
    }
    async setOnlineStatus(status) {
        return await __classPrivateFieldGet$c(this, _NeighbourhoodProxy_client, "f").setOnlineStatus(__classPrivateFieldGet$c(this, _NeighbourhoodProxy_pID, "f"), status);
    }
    async setOnlineStatusU(status) {
        return await __classPrivateFieldGet$c(this, _NeighbourhoodProxy_client, "f").setOnlineStatusU(__classPrivateFieldGet$c(this, _NeighbourhoodProxy_pID, "f"), status);
    }
    async sendSignal(remoteAgentDid, payload) {
        return await __classPrivateFieldGet$c(this, _NeighbourhoodProxy_client, "f").sendSignal(__classPrivateFieldGet$c(this, _NeighbourhoodProxy_pID, "f"), remoteAgentDid, payload);
    }
    async sendSignalU(remoteAgentDid, payload) {
        return await __classPrivateFieldGet$c(this, _NeighbourhoodProxy_client, "f").sendSignalU(__classPrivateFieldGet$c(this, _NeighbourhoodProxy_pID, "f"), remoteAgentDid, payload);
    }
    async sendBroadcast(payload) {
        return await __classPrivateFieldGet$c(this, _NeighbourhoodProxy_client, "f").sendBroadcast(__classPrivateFieldGet$c(this, _NeighbourhoodProxy_pID, "f"), payload);
    }
    async sendBroadcastU(payload) {
        return await __classPrivateFieldGet$c(this, _NeighbourhoodProxy_client, "f").sendBroadcastU(__classPrivateFieldGet$c(this, _NeighbourhoodProxy_pID, "f"), payload);
    }
    async addSignalHandler(handler) {
        await __classPrivateFieldGet$c(this, _NeighbourhoodProxy_client, "f").addSignalHandler(__classPrivateFieldGet$c(this, _NeighbourhoodProxy_pID, "f"), handler);
    }
    removeSignalHandler(handler) {
        __classPrivateFieldGet$c(this, _NeighbourhoodProxy_client, "f").removeSignalHandler(__classPrivateFieldGet$c(this, _NeighbourhoodProxy_pID, "f"), handler);
    }
}
_NeighbourhoodProxy_client = new WeakMap(), _NeighbourhoodProxy_pID = new WeakMap();

var __decorate$7 = function (decorators, target, key, desc) {
    var c = arguments.length, r = c < 3 ? target : desc === null ? desc = Object.getOwnPropertyDescriptor(target, key) : desc, d;
    if (typeof Reflect === "object" && typeof Reflect.decorate === "function") r = Reflect.decorate(decorators, target, key, desc);
    else for (var i = decorators.length - 1; i >= 0; i--) if (d = decorators[i]) r = (c < 3 ? d(r) : c > 3 ? d(target, key, r) : d(target, key)) || r;
    return c > 3 && r && Object.defineProperty(target, key, r), r;
};
var __metadata$7 = function (k, v) {
    if (typeof Reflect === "object" && typeof Reflect.metadata === "function") return Reflect.metadata(k, v);
};
let LinkQuery = class LinkQuery {
    constructor(obj) {
        if (obj) {
            // @ts-ignore
            this.source = obj.source;
            // @ts-ignore
            this.predicate = obj.predicate;
            // @ts-ignore
            this.target = obj.target;
            // @ts-ignore
            if (obj.fromDate) {
                // @ts-ignore
                this.fromDate = obj.fromDate;
            }
            // @ts-ignore
            if (obj.untilDate) {
                // @ts-ignore
                this.untilDate = obj.untilDate;
            }
            // @ts-ignore
            if (obj.limit) {
                // @ts-ignore
                this.limit = obj.limit;
            }
        }
    }
    isMatch(l) {
        if (this.source)
            if (this.source !== l.source)
                return false;
        if (this.predicate)
            if (this.predicate !== l.predicate)
                return false;
        if (this.target)
            if (this.target !== l.target)
                return false;
        return true;
    }
};
__decorate$7([
    Field(),
    __metadata$7("design:type", String)
], LinkQuery.prototype, "source", void 0);
__decorate$7([
    Field(),
    __metadata$7("design:type", String)
], LinkQuery.prototype, "target", void 0);
__decorate$7([
    Field(),
    __metadata$7("design:type", String)
], LinkQuery.prototype, "predicate", void 0);
__decorate$7([
    Field(),
    __metadata$7("design:type", Date)
], LinkQuery.prototype, "fromDate", void 0);
__decorate$7([
    Field(),
    __metadata$7("design:type", Date)
], LinkQuery.prototype, "untilDate", void 0);
__decorate$7([
    Field(),
    __metadata$7("design:type", Number)
], LinkQuery.prototype, "limit", void 0);
LinkQuery = __decorate$7([
    ObjectType(),
    InputType(),
    __metadata$7("design:paramtypes", [Object])
], LinkQuery);

var __classPrivateFieldSet$b = function (receiver, state, value, kind, f) {
    if (kind === "m") throw new TypeError("Private method is not writable");
    if (kind === "a" && !f) throw new TypeError("Private accessor was defined without a setter");
    if (typeof state === "function" ? receiver !== state || !f : !state.has(receiver)) throw new TypeError("Cannot write private member to an object whose class did not declare it");
    return (kind === "a" ? f.call(receiver, value) : f ? f.value = value : state.set(receiver, value)), value;
};
var __classPrivateFieldGet$b = function (receiver, state, kind, f) {
    if (kind === "a" && !f) throw new TypeError("Private accessor was defined without a getter");
    if (typeof state === "function" ? receiver !== state || !f : !state.has(receiver)) throw new TypeError("Cannot read private member from an object whose class did not declare it");
    return kind === "m" ? f : kind === "a" ? f.call(receiver) : f ? f.value : state.get(receiver);
};
var _Literal_literal, _Literal_url;
function encodeRFC3986URIComponent(str) {
    return encodeURIComponent(str)
        .replace(/[!'()*]/g, (c) => `%${c.charCodeAt(0).toString(16).toUpperCase()}`);
}
class Literal {
    constructor() {
        _Literal_literal.set(this, void 0);
        _Literal_url.set(this, void 0);
    }
    static fromUrl(url) {
        if (!url || !url.startsWith("literal://"))
            throw new Error("Can't create Literal from non-literal URL");
        const l = new Literal();
        __classPrivateFieldSet$b(l, _Literal_url, url, "f");
        return l;
    }
    static from(literal) {
        const l = new Literal();
        __classPrivateFieldSet$b(l, _Literal_literal, literal, "f");
        return l;
    }
    toUrl() {
        if (__classPrivateFieldGet$b(this, _Literal_url, "f") && !__classPrivateFieldGet$b(this, _Literal_literal, "f"))
            return __classPrivateFieldGet$b(this, _Literal_url, "f");
        if (!__classPrivateFieldGet$b(this, _Literal_url, "f") && !__classPrivateFieldGet$b(this, _Literal_literal, "f"))
            throw new Error("Can't turn empty Literal into URL");
        let encoded;
        switch (typeof __classPrivateFieldGet$b(this, _Literal_literal, "f")) {
            case 'string':
                encoded = `string:${encodeRFC3986URIComponent(__classPrivateFieldGet$b(this, _Literal_literal, "f"))}`;
                break;
            case 'number':
                encoded = `number:${encodeRFC3986URIComponent(__classPrivateFieldGet$b(this, _Literal_literal, "f"))}`;
                break;
            case 'object':
                encoded = `json:${encodeRFC3986URIComponent(JSON.stringify(__classPrivateFieldGet$b(this, _Literal_literal, "f")))}`;
                break;
        }
        return `literal://${encoded}`;
    }
    get() {
        if (__classPrivateFieldGet$b(this, _Literal_literal, "f"))
            return __classPrivateFieldGet$b(this, _Literal_literal, "f");
        if (!__classPrivateFieldGet$b(this, _Literal_url, "f"))
            throw new Error("Can't render empty Literal");
        if (!__classPrivateFieldGet$b(this, _Literal_url, "f").startsWith("literal://"))
            throw new Error("Can't render Literal from non-literal URL");
        // get rid of "literal://"
        const body = __classPrivateFieldGet$b(this, _Literal_url, "f").substring(10);
        if (body.startsWith("string:")) {
            return decodeURIComponent(body.substring(7));
        }
        if (body.startsWith("number:")) {
            const numberString = body.substring(7);
            return parseFloat(numberString);
        }
        if (body.startsWith("json:")) {
            const json = body.substring(5);
            return JSON.parse(decodeURIComponent(json));
        }
        throw new Error(`Can't parse unknown literal: ${body}`);
    }
}
_Literal_literal = new WeakMap(), _Literal_url = new WeakMap();

function capitalize(str) {
    return str.charAt(0).toUpperCase() + str.slice(1);
}
// e.g. "name" -> "setName"
function propertyNameToSetterName(property) {
    return `set${capitalize(property)}`;
}
function singularToPlural(singular) {
    if (singular.endsWith("y")) {
        return singular.slice(0, -1) + "ies";
    }
    else {
        return singular + "s";
    }
}
function pluralToSingular(plural) {
    if (plural.endsWith("ies")) {
        return plural.slice(0, -3) + "y";
    }
    else if (plural.endsWith("s")) {
        return plural.slice(0, -1);
    }
    else {
        return plural;
    }
}
// e.g. "comments" -> "addComment"
function collectionToAdderName(collection) {
    return `add${capitalize(pluralToSingular(collection))}`;
}
// e.g. "addEntry" -> "entries"
function collectionAdderToName(adderName) {
    let singular = adderName.substring(3);
    let plural = singularToPlural(singular);
    return plural.charAt(0).toLowerCase() + plural.slice(1);
}
// e.g. "comments" -> "removeComment"
function collectionToRemoverName(collection) {
    return `remove${capitalize(pluralToSingular(collection))}`;
}
// e.g. "removeEntry" -> "entries"
function collectionRemoverToName(removerName) {
    let singular = removerName.substring(6);
    let plural = singularToPlural(singular);
    return plural.charAt(0).toLowerCase() + plural.slice(1);
}
function collectionSetterToName(adderName) {
    let singular = adderName.substring(13);
    let plural = singularToPlural(singular);
    return plural.charAt(0).toLowerCase() + plural.slice(1);
}
// e.g. "comments" -> "addComment"
function collectionToSetterName(collection) {
    return `setCollection${capitalize(pluralToSingular(collection))}`;
}

var __classPrivateFieldSet$a = function (receiver, state, value, kind, f) {
    if (kind === "m") throw new TypeError("Private method is not writable");
    if (kind === "a" && !f) throw new TypeError("Private accessor was defined without a setter");
    if (typeof state === "function" ? receiver !== state || !f : !state.has(receiver)) throw new TypeError("Cannot write private member to an object whose class did not declare it");
    return (kind === "a" ? f.call(receiver, value) : f ? f.value = value : state.set(receiver, value)), value;
};
var __classPrivateFieldGet$a = function (receiver, state, kind, f) {
    if (kind === "a" && !f) throw new TypeError("Private accessor was defined without a getter");
    if (typeof state === "function" ? receiver !== state || !f : !state.has(receiver)) throw new TypeError("Cannot read private member from an object whose class did not declare it");
    return kind === "m" ? f : kind === "a" ? f.call(receiver) : f ? f.value : state.get(receiver);
};
var _Subject_baseExpression, _Subject_subjectClass, _Subject_perspective;
class Subject {
    constructor(perspective, baseExpression, subjectClass) {
        _Subject_baseExpression.set(this, void 0);
        _Subject_subjectClass.set(this, void 0);
        _Subject_perspective.set(this, void 0);
        __classPrivateFieldSet$a(this, _Subject_baseExpression, baseExpression, "f");
        __classPrivateFieldSet$a(this, _Subject_subjectClass, subjectClass, "f");
        __classPrivateFieldSet$a(this, _Subject_perspective, perspective, "f");
    }
    get baseExpression() {
        return __classPrivateFieldGet$a(this, _Subject_baseExpression, "f");
    }
    async init() {
        let isInstance = await __classPrivateFieldGet$a(this, _Subject_perspective, "f").isSubjectInstance(__classPrivateFieldGet$a(this, _Subject_baseExpression, "f"), __classPrivateFieldGet$a(this, _Subject_subjectClass, "f"));
        if (!isInstance) {
            throw `Not a valid subject instance of ${__classPrivateFieldGet$a(this, _Subject_subjectClass, "f")} for ${__classPrivateFieldGet$a(this, _Subject_baseExpression, "f")}`;
        }
        let results = await __classPrivateFieldGet$a(this, _Subject_perspective, "f").infer(`subject_class("${__classPrivateFieldGet$a(this, _Subject_subjectClass, "f")}", C), property(C, Property)`);
        let properties = results.map(result => result.Property);
        //console.log("Subject properties: " + properties)
        for (let p of properties) {
            const resolveExpressionURI = await __classPrivateFieldGet$a(this, _Subject_perspective, "f").infer(`subject_class("${__classPrivateFieldGet$a(this, _Subject_subjectClass, "f")}", C), property_resolve(C, "${p}")`);
            Object.defineProperty(this, p, {
                configurable: true,
                get: async () => {
                    let results = await __classPrivateFieldGet$a(this, _Subject_perspective, "f").infer(`subject_class("${__classPrivateFieldGet$a(this, _Subject_subjectClass, "f")}", C), property_getter(C, "${__classPrivateFieldGet$a(this, _Subject_baseExpression, "f")}", "${p}", Value)`);
                    if (results && results.length > 0) {
                        let expressionURI = results[0].Value;
                        if (resolveExpressionURI) {
                            try {
                                const expression = await __classPrivateFieldGet$a(this, _Subject_perspective, "f").getExpression(expressionURI);
                                try {
                                    return JSON.parse(expression.data);
                                }
                                catch (e) {
                                    return expression.data;
                                }
                            }
                            catch (err) {
                                return expressionURI;
                            }
                        }
                        else {
                            return expressionURI;
                        }
                    }
                    else if (results) {
                        return results;
                    }
                    else {
                        return undefined;
                    }
                }
            });
        }
        const setters = await __classPrivateFieldGet$a(this, _Subject_perspective, "f").infer(`subject_class("${__classPrivateFieldGet$a(this, _Subject_subjectClass, "f")}", C), property_setter(C, Property, Setter)`);
        //console.log("Subject setters: " + setters.map(setter => setter.Property))
        for (let setter of (setters ? setters : [])) {
            if (setter) {
                const property = setter.Property;
                const actions = eval(setter.Setter);
                const resolveLanguageResults = await __classPrivateFieldGet$a(this, _Subject_perspective, "f").infer(`subject_class("${__classPrivateFieldGet$a(this, _Subject_subjectClass, "f")}", C), property_resolve_language(C, "${property}", Language)`);
                let resolveLanguage;
                if (resolveLanguageResults && resolveLanguageResults.length > 0) {
                    resolveLanguage = resolveLanguageResults[0].Language;
                }
                this[propertyNameToSetterName(property)] = async (value) => {
                    if (resolveLanguage) {
                        value = await __classPrivateFieldGet$a(this, _Subject_perspective, "f").createExpression(value, resolveLanguage);
                    }
                    await __classPrivateFieldGet$a(this, _Subject_perspective, "f").executeAction(actions, __classPrivateFieldGet$a(this, _Subject_baseExpression, "f"), [{ name: "value", value }]);
                };
            }
        }
        let results2 = await __classPrivateFieldGet$a(this, _Subject_perspective, "f").infer(`subject_class("${__classPrivateFieldGet$a(this, _Subject_subjectClass, "f")}", C), collection(C, Collection)`);
        if (!results2)
            results2 = [];
        let collections = results2.map(result => result.Collection);
        for (let c of collections) {
            Object.defineProperty(this, c, {
                configurable: true,
                get: async () => {
                    let results = await __classPrivateFieldGet$a(this, _Subject_perspective, "f").infer(`subject_class("${__classPrivateFieldGet$a(this, _Subject_subjectClass, "f")}", C), collection_getter(C, "${__classPrivateFieldGet$a(this, _Subject_baseExpression, "f")}", "${c}", Value)`);
                    if (results && results.length > 0 && results[0].Value) {
                        let collectionContent = results[0].Value.filter((v) => v !== "" && v !== '');
                        return collectionContent;
                    }
                    else {
                        return [];
                    }
                }
            });
        }
        let adders = await __classPrivateFieldGet$a(this, _Subject_perspective, "f").infer(`subject_class("${__classPrivateFieldGet$a(this, _Subject_subjectClass, "f")}", C), collection_adder(C, Collection, Adder)`);
        if (!adders)
            adders = [];
        for (let adder of adders) {
            if (adder) {
                const collection = adder.Collection;
                const actions = eval(adder.Adder);
                this[collectionToAdderName(collection)] = async (value) => {
                    if (Array.isArray(value)) {
                        await Promise.all(value.map(v => __classPrivateFieldGet$a(this, _Subject_perspective, "f").executeAction(actions, __classPrivateFieldGet$a(this, _Subject_baseExpression, "f"), [{ name: "value", value: v }])));
                    }
                    else {
                        await __classPrivateFieldGet$a(this, _Subject_perspective, "f").executeAction(actions, __classPrivateFieldGet$a(this, _Subject_baseExpression, "f"), [{ name: "value", value }]);
                    }
                };
            }
        }
        let removers = await __classPrivateFieldGet$a(this, _Subject_perspective, "f").infer(`subject_class("${__classPrivateFieldGet$a(this, _Subject_subjectClass, "f")}", C), collection_remover(C, Collection, Remover)`);
        if (!removers)
            removers = [];
        for (let remover of removers) {
            if (remover) {
                const collection = remover.Collection;
                const actions = eval(remover.Remover);
                this[collectionToRemoverName(collection)] = async (value) => {
                    if (Array.isArray(value)) {
                        await Promise.all(value.map(v => __classPrivateFieldGet$a(this, _Subject_perspective, "f").executeAction(actions, __classPrivateFieldGet$a(this, _Subject_baseExpression, "f"), [{ name: "value", value: v }])));
                    }
                    else {
                        await __classPrivateFieldGet$a(this, _Subject_perspective, "f").executeAction(actions, __classPrivateFieldGet$a(this, _Subject_baseExpression, "f"), [{ name: "value", value }]);
                    }
                };
            }
        }
        let collectionSetters = await __classPrivateFieldGet$a(this, _Subject_perspective, "f").infer(`subject_class("${__classPrivateFieldGet$a(this, _Subject_subjectClass, "f")}", C), collection_setter(C, Collection, Setter)`);
        if (!collectionSetters)
            collectionSetters = [];
        for (let collectionSetter of collectionSetters) {
            if (collectionSetter) {
                const collection = collectionSetter.Collection;
                const actions = eval(collectionSetter.Setter);
                this[collectionToSetterName(collection)] = async (value) => {
                    if (Array.isArray(value)) {
                        await __classPrivateFieldGet$a(this, _Subject_perspective, "f").executeAction(actions, __classPrivateFieldGet$a(this, _Subject_baseExpression, "f"), value.map(v => ({ name: "value", value: v })));
                    }
                    else {
                        await __classPrivateFieldGet$a(this, _Subject_perspective, "f").executeAction(actions, __classPrivateFieldGet$a(this, _Subject_baseExpression, "f"), [{ name: "value", value }]);
                    }
                };
            }
        }
    }
}
_Subject_baseExpression = new WeakMap(), _Subject_subjectClass = new WeakMap(), _Subject_perspective = new WeakMap();

var __classPrivateFieldSet$9 = function (receiver, state, value, kind, f) {
    if (kind === "m") throw new TypeError("Private method is not writable");
    if (kind === "a" && !f) throw new TypeError("Private accessor was defined without a setter");
    if (typeof state === "function" ? receiver !== state || !f : !state.has(receiver)) throw new TypeError("Cannot write private member to an object whose class did not declare it");
    return (kind === "a" ? f.call(receiver, value) : f ? f.value = value : state.set(receiver, value)), value;
};
var __classPrivateFieldGet$9 = function (receiver, state, kind, f) {
    if (kind === "a" && !f) throw new TypeError("Private accessor was defined without a getter");
    if (typeof state === "function" ? receiver !== state || !f : !state.has(receiver)) throw new TypeError("Cannot read private member from an object whose class did not declare it");
    return kind === "m" ? f : kind === "a" ? f.call(receiver) : f ? f.value : state.get(receiver);
};
var _PerspectiveProxy_handle, _PerspectiveProxy_client, _PerspectiveProxy_perspectiveLinkAddedCallbacks, _PerspectiveProxy_perspectiveLinkRemovedCallbacks, _PerspectiveProxy_perspectiveLinkUpdatedCallbacks, _PerspectiveProxy_perspectiveSyncStateChangeCallbacks;
/** Perspective UI proxy object
 *
 * Convenience object for UIs to interact with a perspective.
 * It is created by some of the methods in the PerspectiveClient class and includes
 * a reference to the PerspectiveClient object that created it.
 */
class PerspectiveProxy {
    constructor(handle, ad4m) {
        _PerspectiveProxy_handle.set(this, void 0);
        _PerspectiveProxy_client.set(this, void 0);
        _PerspectiveProxy_perspectiveLinkAddedCallbacks.set(this, void 0);
        _PerspectiveProxy_perspectiveLinkRemovedCallbacks.set(this, void 0);
        _PerspectiveProxy_perspectiveLinkUpdatedCallbacks.set(this, void 0);
        _PerspectiveProxy_perspectiveSyncStateChangeCallbacks.set(this, void 0);
        __classPrivateFieldSet$9(this, _PerspectiveProxy_perspectiveLinkAddedCallbacks, [], "f");
        __classPrivateFieldSet$9(this, _PerspectiveProxy_perspectiveLinkRemovedCallbacks, [], "f");
        __classPrivateFieldSet$9(this, _PerspectiveProxy_perspectiveLinkUpdatedCallbacks, [], "f");
        __classPrivateFieldSet$9(this, _PerspectiveProxy_perspectiveSyncStateChangeCallbacks, [], "f");
        __classPrivateFieldSet$9(this, _PerspectiveProxy_handle, handle, "f");
        __classPrivateFieldSet$9(this, _PerspectiveProxy_client, ad4m, "f");
        __classPrivateFieldGet$9(this, _PerspectiveProxy_client, "f").addPerspectiveLinkAddedListener(__classPrivateFieldGet$9(this, _PerspectiveProxy_handle, "f").uuid, __classPrivateFieldGet$9(this, _PerspectiveProxy_perspectiveLinkAddedCallbacks, "f"));
        __classPrivateFieldGet$9(this, _PerspectiveProxy_client, "f").addPerspectiveLinkRemovedListener(__classPrivateFieldGet$9(this, _PerspectiveProxy_handle, "f").uuid, __classPrivateFieldGet$9(this, _PerspectiveProxy_perspectiveLinkRemovedCallbacks, "f"));
        __classPrivateFieldGet$9(this, _PerspectiveProxy_client, "f").addPerspectiveLinkUpdatedListener(__classPrivateFieldGet$9(this, _PerspectiveProxy_handle, "f").uuid, __classPrivateFieldGet$9(this, _PerspectiveProxy_perspectiveLinkUpdatedCallbacks, "f"));
        __classPrivateFieldGet$9(this, _PerspectiveProxy_client, "f").addPerspectiveSyncStateChangeListener(__classPrivateFieldGet$9(this, _PerspectiveProxy_handle, "f").uuid, __classPrivateFieldGet$9(this, _PerspectiveProxy_perspectiveSyncStateChangeCallbacks, "f"));
    }
    async executeAction(actions, expression, parameters) {
        const replaceThis = (input) => {
            if (input) {
                if (input === 'this') {
                    return expression;
                }
                else {
                    return input;
                }
            }
            else {
                return undefined;
            }
        };
        const replaceParameters = (input) => {
            if (parameters) {
                let output = input;
                for (const parameter of parameters) {
                    output = output.replace(parameter.name, parameter.value);
                }
                return output;
            }
            else
                return input;
        };
        for (let command of actions) {
            let source = replaceThis(replaceParameters(command.source));
            let predicate = replaceThis(replaceParameters(command.predicate));
            let target = replaceThis(replaceParameters(command.target));
            let local = command?.local ?? false;
            switch (command.action) {
                case 'addLink':
                    await this.add(new Link({ source, predicate, target }), local ? 'local' : 'shared');
                    break;
                case 'removeLink':
                    const linkExpressions = await this.get(new LinkQuery({ source, predicate, target }));
                    for (const linkExpression of linkExpressions) {
                        await this.remove(linkExpression);
                    }
                    break;
                case 'setSingleTarget':
                    await this.setSingleTarget(new Link({ source, predicate, target }), local ? 'local' : 'shared');
                    break;
                case 'collectionSetter':
                    const links = await this.get(new LinkQuery({ source, predicate }));
                    await this.removeLinks(links);
                    await this.addLinks(parameters.map(p => new Link({ source, predicate, target: p.value })), local ? 'local' : 'shared');
                    break;
            }
        }
    }
    /** Unique ID of the perspective */
    get uuid() {
        return __classPrivateFieldGet$9(this, _PerspectiveProxy_handle, "f").uuid;
    }
    /** Given name of the perspective */
    get name() {
        return __classPrivateFieldGet$9(this, _PerspectiveProxy_handle, "f").name;
    }
    /** If the perspective is shared as a Neighbourhood, this is the Neighbourhood URL */
    get sharedUrl() {
        return __classPrivateFieldGet$9(this, _PerspectiveProxy_handle, "f").sharedUrl;
    }
    /** If the perspective is shared as a Neighbourhood, this is the Neighbourhood Expression */
    get neighbourhood() {
        return __classPrivateFieldGet$9(this, _PerspectiveProxy_handle, "f").neighbourhood;
    }
    /** Returns the state of the perspective **/
    get state() {
        return __classPrivateFieldGet$9(this, _PerspectiveProxy_handle, "f").state;
    }
    /** Returns all the links of this perspective that matches the LinkQuery */
    async get(query) {
        return await __classPrivateFieldGet$9(this, _PerspectiveProxy_client, "f").queryLinks(__classPrivateFieldGet$9(this, _PerspectiveProxy_handle, "f").uuid, query);
    }
    /** Runs a Prolog query on the perspective's Prolog engine */
    async infer(query) {
        return await __classPrivateFieldGet$9(this, _PerspectiveProxy_client, "f").queryProlog(__classPrivateFieldGet$9(this, _PerspectiveProxy_handle, "f").uuid, query);
    }
    /** Adds a link to this perspective */
    async add(link, status = 'shared') {
        return await __classPrivateFieldGet$9(this, _PerspectiveProxy_client, "f").addLink(__classPrivateFieldGet$9(this, _PerspectiveProxy_handle, "f").uuid, link, status);
    }
    /** Adds multiple links to this perspective **/
    async addLinks(links, status = 'shared') {
        return await __classPrivateFieldGet$9(this, _PerspectiveProxy_client, "f").addLinks(__classPrivateFieldGet$9(this, _PerspectiveProxy_handle, "f").uuid, links, status);
    }
    /** Removes multiple links from this perspective **/
    async removeLinks(links) {
        return await __classPrivateFieldGet$9(this, _PerspectiveProxy_client, "f").removeLinks(__classPrivateFieldGet$9(this, _PerspectiveProxy_handle, "f").uuid, links);
    }
    /** Adds and removes multiple links from this perspective **/
    async linkMutations(mutations, status = 'shared') {
        return await __classPrivateFieldGet$9(this, _PerspectiveProxy_client, "f").linkMutations(__classPrivateFieldGet$9(this, _PerspectiveProxy_handle, "f").uuid, mutations, status);
    }
    /** Adds a linkExpression to this perspective */
    async addLinkExpression(link, status = 'shared') {
        return await __classPrivateFieldGet$9(this, _PerspectiveProxy_client, "f").addLinkExpression(__classPrivateFieldGet$9(this, _PerspectiveProxy_handle, "f").uuid, link, status);
    }
    async update(oldLink, newLink) {
        return await __classPrivateFieldGet$9(this, _PerspectiveProxy_client, "f").updateLink(__classPrivateFieldGet$9(this, _PerspectiveProxy_handle, "f").uuid, oldLink, newLink);
    }
    async remove(link) {
        return await __classPrivateFieldGet$9(this, _PerspectiveProxy_client, "f").removeLink(__classPrivateFieldGet$9(this, _PerspectiveProxy_handle, "f").uuid, link);
    }
    async getExpression(expressionURI) {
        return await __classPrivateFieldGet$9(this, _PerspectiveProxy_client, "f").getExpression(expressionURI);
    }
    async createExpression(content, languageAddress) {
        return await __classPrivateFieldGet$9(this, _PerspectiveProxy_client, "f").createExpression(content, languageAddress);
    }
    /** Adds a link listener
     * @param type Can be 'link-added' or 'link-removed'
     * @param cb Callback function that is called when a link is added to the perspective
     */
    async addListener(type, cb) {
        if (type === 'link-added') {
            __classPrivateFieldGet$9(this, _PerspectiveProxy_perspectiveLinkAddedCallbacks, "f").push(cb);
        }
        else if (type === 'link-removed') {
            __classPrivateFieldGet$9(this, _PerspectiveProxy_perspectiveLinkRemovedCallbacks, "f").push(cb);
        }
        else if (type === 'link-updated') {
            __classPrivateFieldGet$9(this, _PerspectiveProxy_perspectiveLinkUpdatedCallbacks, "f").push(cb);
        }
    }
    /** Adds a sync state listener
     * @param cb Callback function that is called when the sync state of the perspective changes
     * @returns A function that can be called to remove the listener
     */
    async addSyncStateChangeListener(cb) {
        __classPrivateFieldGet$9(this, _PerspectiveProxy_perspectiveSyncStateChangeCallbacks, "f").push(cb);
    }
    /** Removes a previously added link listener
     * @param type Can be 'link-added' or 'link-removed'
     * @param cb Callback function that is called when a link is added to the perspective
     */
    async removeListener(type, cb) {
        if (type === 'link-added') {
            const index = __classPrivateFieldGet$9(this, _PerspectiveProxy_perspectiveLinkAddedCallbacks, "f").indexOf(cb);
            __classPrivateFieldGet$9(this, _PerspectiveProxy_perspectiveLinkAddedCallbacks, "f").splice(index, 1);
        }
        else if (type === 'link-removed') {
            const index = __classPrivateFieldGet$9(this, _PerspectiveProxy_perspectiveLinkRemovedCallbacks, "f").indexOf(cb);
            __classPrivateFieldGet$9(this, _PerspectiveProxy_perspectiveLinkRemovedCallbacks, "f").splice(index, 1);
        }
        else if (type === 'link-updated') {
            const index = __classPrivateFieldGet$9(this, _PerspectiveProxy_perspectiveLinkUpdatedCallbacks, "f").indexOf(cb);
            __classPrivateFieldGet$9(this, _PerspectiveProxy_perspectiveLinkUpdatedCallbacks, "f").splice(index, 1);
        }
    }
    /** Create and return a snapshot of this perspective
     * A snapshot is a rendered Perspectie object that contains all the links of the perspective.
     */
    async snapshot() {
        return __classPrivateFieldGet$9(this, _PerspectiveProxy_client, "f").snapshotByUUID(__classPrivateFieldGet$9(this, _PerspectiveProxy_handle, "f").uuid);
    }
    /** Take and load all the links from the given snapshot */
    async loadSnapshot(snapshot) {
        //Clean the input data from __typename
        const cleanedSnapshot = JSON.parse(JSON.stringify(snapshot));
        delete cleanedSnapshot.__typename;
        cleanedSnapshot.links.forEach(link => {
            delete link.data.__typename;
        });
        for (const link of cleanedSnapshot.links) {
            await this.addLinkExpression(link);
        }
    }
    /** Convenience function to get the target of the first link that matches the given query
     * This makes sense when the query is expected to return only one link
     * and the target of that link is what you are looking for.
     *
     * Works best together with @member setSingelTarget()
     */
    async getSingleTarget(query) {
        delete query.target;
        const foundLinks = await this.get(query);
        if (foundLinks.length)
            return foundLinks[0].data.target;
        else
            return null;
    }
    /** Convenience function to ensure there is only one link with given source and predicate
     * This function will remove all links with the same source and predicate as the given link,
     * and then add the given link.
     * This ensures there is only one target for the given source and predicate.
     *
     * Works best together with @member getSingleTarget()
     */
    async setSingleTarget(link, status = 'shared') {
        const query = new LinkQuery({ source: link.source, predicate: link.predicate });
        const foundLinks = await this.get(query);
        const removals = [];
        for (const l of foundLinks) {
            delete l.__typename;
            delete l.data.__typename;
            delete l.proof.__typename;
            removals.push(l);
        }
        const additions = [link];
        await this.linkMutations({ additions, removals }, status);
    }
    /** Returns all the Social DNA flows defined in this perspective */
    async sdnaFlows() {
        const allFlows = await this.infer("register_sdna_flow(X, _)");
        return allFlows.map(x => x.X);
    }
    /** Returns all Social DNA flows that can be started from the given expression */
    async availableFlows(exprAddr) {
        const availableFlows = await this.infer(`flowable("${exprAddr}", F), register_sdna_flow(X, F)`);
        return availableFlows.map(x => x.X);
    }
    /**  Starts the Social DNA flow @param flowName on the expression @param exprAddr */
    async startFlow(flowName, exprAddr) {
        let startAction = await this.infer(`start_action(Action, F), register_sdna_flow("${flowName}", F)`);
        // should always return one solution...
        startAction = eval(startAction[0].Action);
        await this.executeAction(startAction, exprAddr, undefined);
    }
    /** Returns all expressions in the given state of given Social DNA flow */
    async expressionsInFlowState(flowName, flowState) {
        let expressions = await this.infer(`register_sdna_flow("${flowName}", F), flow_state(X, ${flowState}, F)`);
        return expressions.map(r => r.X);
    }
    /** Returns the given expression's flow state with regard to given Social DNA flow */
    async flowState(flowName, exprAddr) {
        let state = await this.infer(`register_sdna_flow("${flowName}", F), flow_state("${exprAddr}", X, F)`);
        return state[0].X;
    }
    /** Returns available action names, with regard to Social DNA flow and expression's flow state */
    async flowActions(flowName, exprAddr) {
        let actionNames = await this.infer(`register_sdna_flow("${flowName}", Flow), flow_state("${exprAddr}", State, Flow), action(State, Name, _, _)`);
        return actionNames.map(r => r.Name);
    }
    /** Runs given Social DNA flow action */
    async runFlowAction(flowName, exprAddr, actionName) {
        let action = await this.infer(`register_sdna_flow("${flowName}", Flow), flow_state("${exprAddr}", State, Flow), action(State, "${actionName}", _, Action)`);
        // should find only one
        action = eval(action[0].Action);
        await this.executeAction(action, exprAddr, undefined);
    }
    /** Returns the perspective's Social DNA code
     * This will return all SDNA code elements in an array.
     */
    async getSdna() {
        let links = await this.get(new LinkQuery({
            predicate: "ad4m://sdna"
        }));
        return links.map(link => link.data.target).map(t => Literal.fromUrl(t).get());
    }
    /** Adds the given Social DNA code to the perspective's SDNA code */
    async addSdna(name, sdnaCode, type) {
        let predicate = "ad4m://has_custom_sdna";
        if (type === 'subject_class')
            predicate = "ad4m://has_subject_class";
        else if (type === 'flow')
            predicate = "ad4m://has_flow";
        const literalName = Literal.from(name).toUrl();
        const links = await this.get(new LinkQuery({
            source: "ad4m://self",
            predicate,
            target: literalName
        }));
        const sdnaLinks = [];
        if (links.length === 0) {
            sdnaLinks.push(new Link({
                source: "ad4m://self",
                predicate,
                target: literalName
            }));
        }
        sdnaLinks.push(new Link({
            source: literalName,
            predicate: "ad4m://sdna",
            target: Literal.from(sdnaCode).toUrl()
        }));
        await this.addLinks(sdnaLinks);
    }
    /** Returns all the Subject classes defined in this perspectives SDNA */
    async subjectClasses() {
        try {
            return (await this.infer("subject_class(X, _)")).map(x => x.X);
        }
        catch (e) {
            return [];
        }
    }
    async stringOrTemplateObjectToSubjectClass(subjectClass) {
        if (typeof subjectClass === "string")
            return subjectClass;
        else {
            let subjectClasses = await this.subjectClassesByTemplate(subjectClass);
            if (subjectClasses[0]) {
                return subjectClasses[0];
            }
            else {
                //@ts-ignore
                return subjectClass.className;
            }
        }
    }
    /** Creates a new subject instance by running its (SDNA defined) constructor,
     * which means adding links around the given expression address so that it
     * conforms to the given subject class.
     *
     * @param subjectClass Either a string with the name of the subject class, or an object
     * with the properties of the subject class. In the latter case, the first subject class
     * that matches the given properties will be used.
     * @param exprAddr The address of the expression to be turned into a subject instance
     */
    async createSubject(subjectClass, exprAddr) {
        let className = await this.stringOrTemplateObjectToSubjectClass(subjectClass);
        let result = await this.infer(`subject_class("${className}", C), constructor(C, Actions)`);
        if (!result.length) {
            throw "No constructor found for given subject class: " + className;
        }
        let actions = result.map(x => eval(x.Actions));
        await this.executeAction(actions[0], exprAddr, undefined);
        return this.getSubjectProxy(exprAddr, subjectClass);
    }
    /** Removes a subject instance by running its (SDNA defined) destructor,
     * which means removing links around the given expression address
     *
     * @param subjectClass Either a string with the name of the subject class, or an object
     * with the properties of the subject class. In the latter case, the first subject class
     * that matches the given properties will be used.
     * @param exprAddr The address of the expression to be turned into a subject instance
     */
    async removeSubject(subjectClass, exprAddr) {
        let className = await this.stringOrTemplateObjectToSubjectClass(subjectClass);
        let result = await this.infer(`subject_class("${className}", C), destructor(C, Actions)`);
        if (!result.length) {
            throw "No constructor found for given subject class: " + className;
        }
        let actions = result.map(x => eval(x.Actions));
        await this.executeAction(actions[0], exprAddr, undefined);
    }
    /** Checks if the given expression is a subject instance of the given subject class
     * @param expression The expression to be checked
     * @param subjectClass Either a string with the name of the subject class, or an object
     * with the properties of the subject class. In the latter case, the first subject class
     * that matches the given properties will be used.
    */
    async isSubjectInstance(expression, subjectClass) {
        let className = await this.stringOrTemplateObjectToSubjectClass(subjectClass);
        return await this.infer(`subject_class("${className}", C), instance(C, "${expression}")`);
    }
    /** For an existing subject instance (existing in the perspective's links)
     * this function returns a proxy object that can be used to access the subject's
     * properties and methods.
     *
     * @param base URI of the subject's root expression
     * @param subjectClass Either a string with the name of the subject class, or an object
     * with the properties of the subject class. In the latter case, the first subject class
     * that matches the given properties will be used.
     */
    async getSubjectProxy(base, subjectClass) {
        if (!await this.isSubjectInstance(base, subjectClass)) {
            throw `Expression ${base} is not a subject instance of given class: ${JSON.stringify(subjectClass)}`;
        }
        let className = await this.stringOrTemplateObjectToSubjectClass(subjectClass);
        let subject = new Subject(this, base, className);
        await subject.init();
        return subject;
    }
    /** Returns all subject instances of the given subject class as proxy objects.
     *  @param subjectClass Either a string with the name of the subject class, or an object
     * with the properties of the subject class. In the latter case, all subject classes
     * that match the given properties will be used.
     */
    async getAllSubjectInstances(subjectClass) {
        let classes = [];
        if (typeof subjectClass === "string") {
            classes = [subjectClass];
        }
        else {
            classes = await this.subjectClassesByTemplate(subjectClass);
        }
        let instances = [];
        for (let className of classes) {
            let instanceBaseExpressions = await this.infer(`subject_class("${className}", C), instance(C, X)`);
            let newInstances = await Promise.all(instanceBaseExpressions.map(async (x) => await this.getSubjectProxy(x.X, className)));
            instances = instances.concat(newInstances);
        }
        return instances;
    }
    async getAllSubjectProxies(subjectClass) {
        let classes = [];
        if (typeof subjectClass === "string") {
            classes = [subjectClass];
        }
        else {
            classes = await this.subjectClassesByTemplate(subjectClass);
        }
        let instances = [];
        for (let className of classes) {
            instances = await this.infer(`subject_class("${className}", C), instance(C, X)`);
        }
        return instances;
    }
    /** Returns all subject classes that match the given template object.
     * This function looks at the properties of the template object and
     * its setters and collections to create a Prolog query that finds
     * all subject classes that would be converted to a proxy object
     * with exactly the same properties and collections.
     *
     * Since there could be multiple subject classes that match the given
     * criteria, this function returns a list of class names.
     *
     * @param obj The template object
     */
    async subjectClassesByTemplate(obj) {
        // Collect all string properties of the object in a list
        let properties = [];
        // Collect all collections of the object in a list
        let collections = [];
        // Collect all string properties of the object in a list
        if (Object.getPrototypeOf(obj).__properties) {
            Object.keys(Object.getPrototypeOf(obj).__properties).forEach(p => properties.push(p));
        }
        else {
            properties.push(...Object.keys(obj).filter(key => !Array.isArray(obj[key])));
        }
        // Collect all collections of the object in a list
        if (Object.getPrototypeOf(obj).__collections) {
            Object.keys(Object.getPrototypeOf(obj).__collections).filter(key => key !== 'isSubjectInstance').forEach(c => !collections.includes(c) ?? collections.push(c));
        }
        else {
            collections.push(...Object.keys(obj).filter(key => Array.isArray(obj[key])).filter(key => key !== 'isSubjectInstance'));
        }
        // Collect all set functions of the object in a list
        let setFunctions = Object.getOwnPropertyNames(obj).filter(key => (typeof obj[key] === "function") && key.startsWith("set") && !key.startsWith("setCollection"));
        // Add all set functions of the object's prototype to that list
        setFunctions = setFunctions.concat(Object.getOwnPropertyNames(Object.getPrototypeOf(obj)).filter(key => (typeof obj[key] === "function") && key.startsWith("set") && !key.startsWith("setCollection")));
        // Collect all add functions of the object in a list
        let addFunctions = Object.getOwnPropertyNames(obj).filter(key => (typeof obj[key] === "function") && key.startsWith("add"));
        // Add all add functions of the object's prototype to that list
        addFunctions = addFunctions.concat(Object.getOwnPropertyNames(Object.getPrototypeOf(obj)).filter(key => (typeof obj[key] === "function") && key.startsWith("add")));
        // Collect all remove functions of the object in a list
        let removeFunctions = Object.getOwnPropertyNames(obj).filter(key => (typeof obj[key] === "function") && key.startsWith("remove"));
        // Add all remove functions of the object's prototype to that list
        removeFunctions = removeFunctions.concat(Object.getOwnPropertyNames(Object.getPrototypeOf(obj)).filter(key => (typeof obj[key] === "function") && key.startsWith("remove")));
        // Collect all add functions of the object in a list
        let setCollectionFunctions = Object.getOwnPropertyNames(obj).filter(key => (typeof obj[key] === "function") && key.startsWith("setCollection"));
        // Add all add functions of the object's prototype to that list
        setCollectionFunctions = setCollectionFunctions.concat(Object.getOwnPropertyNames(Object.getPrototypeOf(obj)).filter(key => (typeof obj[key] === "function") && key.startsWith("setCollection")));
        // Construct query to find all subject classes that have the given properties and collections
        let query = `subject_class(Class, C)`;
        for (let property of properties) {
            query += `, property(C, "${property}")`;
        }
        for (let collection of collections) {
            query += `, collection(C, "${collection}")`;
        }
        for (let setFunction of setFunctions) {
            // e.g.  "setState" -> "state"
            let property = setFunction.substring(3);
            property = property.charAt(0).toLowerCase() + property.slice(1);
            query += `, property_setter(C, "${property}", _)`;
        }
        for (let addFunction of addFunctions) {
            query += `, collection_adder(C, "${collectionAdderToName(addFunction)}", _)`;
        }
        for (let removeFunction of removeFunctions) {
            query += `, collection_remover(C, "${collectionRemoverToName(removeFunction)}", _)`;
        }
        for (let setCollectionFunction of setCollectionFunctions) {
            query += `, collection_setter(C, "${collectionSetterToName(setCollectionFunction)}", _)`;
        }
        query += ".";
        let result = await this.infer(query);
        if (!result) {
            return [];
        }
        else {
            return result.map(x => x.Class);
        }
    }
    /** Takes a JS class (its constructor) and assumes that it was decorated by
     * the @subjectClass etc. decorators. It then tests if there is a subject class
     * already present in the perspective's SDNA that matches the given class.
     * If there is no such class, it gets the JS class's SDNA by calling its
     * static generateSDNA() function and adds it to the perspective's SDNA.
     */
    async ensureSDNASubjectClass(jsClass) {
        const subjectClass = await this.subjectClassesByTemplate(new jsClass);
        if (subjectClass.length > 0) {
            return;
        }
        const { name, sdna } = jsClass.generateSDNA();
        await this.addSdna(name, sdna, 'subject_class');
    }
    getNeighbourhoodProxy() {
        return __classPrivateFieldGet$9(this, _PerspectiveProxy_client, "f").getNeighbourhoodProxy(__classPrivateFieldGet$9(this, _PerspectiveProxy_handle, "f").uuid);
    }
}
_PerspectiveProxy_handle = new WeakMap(), _PerspectiveProxy_client = new WeakMap(), _PerspectiveProxy_perspectiveLinkAddedCallbacks = new WeakMap(), _PerspectiveProxy_perspectiveLinkRemovedCallbacks = new WeakMap(), _PerspectiveProxy_perspectiveLinkUpdatedCallbacks = new WeakMap(), _PerspectiveProxy_perspectiveSyncStateChangeCallbacks = new WeakMap();

var __classPrivateFieldSet$8 = function (receiver, state, value, kind, f) {
    if (kind === "m") throw new TypeError("Private method is not writable");
    if (kind === "a" && !f) throw new TypeError("Private accessor was defined without a setter");
    if (typeof state === "function" ? receiver !== state || !f : !state.has(receiver)) throw new TypeError("Cannot write private member to an object whose class did not declare it");
    return (kind === "a" ? f.call(receiver, value) : f ? f.value = value : state.set(receiver, value)), value;
};
var __classPrivateFieldGet$8 = function (receiver, state, kind, f) {
    if (kind === "a" && !f) throw new TypeError("Private accessor was defined without a getter");
    if (typeof state === "function" ? receiver !== state || !f : !state.has(receiver)) throw new TypeError("Cannot read private member from an object whose class did not declare it");
    return kind === "m" ? f : kind === "a" ? f.call(receiver) : f ? f.value : state.get(receiver);
};
var _PerspectiveClient_apolloClient, _PerspectiveClient_perspectiveAddedCallbacks, _PerspectiveClient_perspectiveUpdatedCallbacks, _PerspectiveClient_perspectiveRemovedCallbacks, _PerspectiveClient_perspectiveSyncStateChangeCallbacks, _PerspectiveClient_expressionClient, _PerspectiveClient_neighbourhoodClient;
const LINK_EXPRESSION_FIELDS = `
author
timestamp
status
data { source, predicate, target }
proof { valid, invalid, signature, key }
`;
const PERSPECTIVE_HANDLE_FIELDS = `
uuid
name
sharedUrl
state
neighbourhood {
    data {
        linkLanguage
        meta {
            links
                {
                    author
                    timestamp
                    data { source, predicate, target }
                    proof { valid, invalid, signature, key }
                }
        }
    }
    author
}
`;
class PerspectiveClient {
    constructor(client, subscribe = true) {
        _PerspectiveClient_apolloClient.set(this, void 0);
        _PerspectiveClient_perspectiveAddedCallbacks.set(this, void 0);
        _PerspectiveClient_perspectiveUpdatedCallbacks.set(this, void 0);
        _PerspectiveClient_perspectiveRemovedCallbacks.set(this, void 0);
        _PerspectiveClient_perspectiveSyncStateChangeCallbacks.set(this, void 0);
        _PerspectiveClient_expressionClient.set(this, void 0);
        _PerspectiveClient_neighbourhoodClient.set(this, void 0);
        __classPrivateFieldSet$8(this, _PerspectiveClient_apolloClient, client, "f");
        __classPrivateFieldSet$8(this, _PerspectiveClient_perspectiveAddedCallbacks, [], "f");
        __classPrivateFieldSet$8(this, _PerspectiveClient_perspectiveUpdatedCallbacks, [], "f");
        __classPrivateFieldSet$8(this, _PerspectiveClient_perspectiveRemovedCallbacks, [], "f");
        __classPrivateFieldSet$8(this, _PerspectiveClient_perspectiveSyncStateChangeCallbacks, [], "f");
        if (subscribe) {
            this.subscribePerspectiveAdded();
            this.subscribePerspectiveUpdated();
            this.subscribePerspectiveRemoved();
        }
    }
    setExpressionClient(client) {
        __classPrivateFieldSet$8(this, _PerspectiveClient_expressionClient, client, "f");
    }
    setNeighbourhoodClient(client) {
        __classPrivateFieldSet$8(this, _PerspectiveClient_neighbourhoodClient, client, "f");
    }
    async all() {
        const { perspectives } = unwrapApolloResult(await __classPrivateFieldGet$8(this, _PerspectiveClient_apolloClient, "f").query({
            query: gql `query perspectives {
                perspectives {
                    ${PERSPECTIVE_HANDLE_FIELDS}
                }
            }`
        }));
        return perspectives.map(handle => new PerspectiveProxy(handle, this));
    }
    async byUUID(uuid) {
        const { perspective } = unwrapApolloResult(await __classPrivateFieldGet$8(this, _PerspectiveClient_apolloClient, "f").query({
            query: gql `query perspective($uuid: String!) {
                perspective(uuid: $uuid) {
                    ${PERSPECTIVE_HANDLE_FIELDS}
                }
            }`,
            variables: { uuid }
        }));
        if (!perspective)
            return null;
        return new PerspectiveProxy(perspective, this);
    }
    async snapshotByUUID(uuid) {
        const { perspectiveSnapshot } = unwrapApolloResult(await __classPrivateFieldGet$8(this, _PerspectiveClient_apolloClient, "f").query({
            query: gql `query perspectiveSnapshot($uuid: String!) {
                perspectiveSnapshot(uuid: $uuid) {
                    links { ${LINK_EXPRESSION_FIELDS} }
                }
            }`,
            variables: { uuid }
        }));
        return perspectiveSnapshot;
    }
    async publishSnapshotByUUID(uuid) {
        const { perspectivePublishSnapshot } = unwrapApolloResult(await __classPrivateFieldGet$8(this, _PerspectiveClient_apolloClient, "f").mutate({
            mutation: gql `mutation perspectivePublishSnapshot($uuid: String!) {
                perspectivePublishSnapshot(uuid: $uuid)
            }`,
            variables: { uuid }
        }));
        return perspectivePublishSnapshot;
    }
    async queryLinks(uuid, query) {
        const { perspectiveQueryLinks } = unwrapApolloResult(await __classPrivateFieldGet$8(this, _PerspectiveClient_apolloClient, "f").query({
            query: gql `query perspectiveQueryLinks($uuid: String!, $query: LinkQuery!) {
                perspectiveQueryLinks(query: $query, uuid: $uuid) {
                    ${LINK_EXPRESSION_FIELDS}
                }
            }`,
            variables: { uuid, query }
        }));
        return perspectiveQueryLinks;
    }
    async queryProlog(uuid, query) {
        const { perspectiveQueryProlog } = unwrapApolloResult(await __classPrivateFieldGet$8(this, _PerspectiveClient_apolloClient, "f").query({
            query: gql `query perspectiveQueryProlog($uuid: String!, $query: String!) {
                perspectiveQueryProlog(uuid: $uuid, query: $query)
            }`,
            variables: { uuid, query }
        }));
        return JSON.parse(perspectiveQueryProlog);
    }
    async add(name) {
        const { perspectiveAdd } = unwrapApolloResult(await __classPrivateFieldGet$8(this, _PerspectiveClient_apolloClient, "f").mutate({
            mutation: gql `mutation perspectiveAdd($name: String!) {
                perspectiveAdd(name: $name) {
                    ${PERSPECTIVE_HANDLE_FIELDS}
                }
            }`,
            variables: { name }
        }));
        return new PerspectiveProxy(perspectiveAdd, this);
    }
    async update(uuid, name) {
        const { perspectiveUpdate } = unwrapApolloResult(await __classPrivateFieldGet$8(this, _PerspectiveClient_apolloClient, "f").mutate({
            mutation: gql `mutation perspectiveUpdate($uuid: String!, $name: String!) {
                perspectiveUpdate(uuid: $uuid, name: $name) {
                    ${PERSPECTIVE_HANDLE_FIELDS}
                }
            }`,
            variables: { uuid, name }
        }));
        return new PerspectiveProxy(perspectiveUpdate, this);
    }
    async remove(uuid) {
        return unwrapApolloResult(await __classPrivateFieldGet$8(this, _PerspectiveClient_apolloClient, "f").mutate({
            mutation: gql `mutation perspectiveRemove($uuid: String!) {
                perspectiveRemove(uuid: $uuid)
            }`,
            variables: { uuid }
        }));
    }
    async addLink(uuid, link, status) {
        const { perspectiveAddLink } = unwrapApolloResult(await __classPrivateFieldGet$8(this, _PerspectiveClient_apolloClient, "f").mutate({
            mutation: gql `mutation perspectiveAddLink($uuid: String!, $link: LinkInput!, $status: String){
                perspectiveAddLink(link: $link, uuid: $uuid, status: $status) {
                    ${LINK_EXPRESSION_FIELDS}
                }
            }`,
            variables: { uuid, link, status }
        }));
        return perspectiveAddLink;
    }
    async addLinks(uuid, links, status) {
        const { perspectiveAddLinks } = unwrapApolloResult(await __classPrivateFieldGet$8(this, _PerspectiveClient_apolloClient, "f").mutate({
            mutation: gql `mutation perspectiveAddLinks($uuid: String!, $links: [LinkInput!]!, $status: String){
                perspectiveAddLinks(links: $links, uuid: $uuid, status: $status) {
                    ${LINK_EXPRESSION_FIELDS}
                }
            }`,
            variables: { uuid, links, status }
        }));
        return perspectiveAddLinks;
    }
    async removeLinks(uuid, links) {
        const { perspectiveRemoveLinks } = unwrapApolloResult(await __classPrivateFieldGet$8(this, _PerspectiveClient_apolloClient, "f").mutate({
            mutation: gql `mutation perspectiveRemoveLinks($uuid: String!, $links: [LinkExpressionInput!]!){
                perspectiveRemoveLinks(links: $links, uuid: $uuid) {
                    ${LINK_EXPRESSION_FIELDS}
                }
            }`,
            variables: { uuid, links }
        }));
        return perspectiveRemoveLinks;
    }
    async linkMutations(uuid, mutations, status) {
        const { perspectiveLinkMutations } = unwrapApolloResult(await __classPrivateFieldGet$8(this, _PerspectiveClient_apolloClient, "f").mutate({
            mutation: gql `mutation perspectiveLinkMutations($uuid: String!, $mutations: LinkMutations!, $status: String){
                perspectiveLinkMutations(mutations: $mutations, uuid: $uuid, status: $status) {
                    additions {
                        ${LINK_EXPRESSION_FIELDS}
                    }
                    removals {
                        ${LINK_EXPRESSION_FIELDS}
                    }
                }
            }`,
            variables: { uuid, mutations, status }
        }));
        return perspectiveLinkMutations;
    }
    async addLinkExpression(uuid, link, status) {
        const { perspectiveAddLinkExpression } = unwrapApolloResult(await __classPrivateFieldGet$8(this, _PerspectiveClient_apolloClient, "f").mutate({
            mutation: gql `mutation perspectiveAddLinkExpression($uuid: String!, $link: LinkExpressionInput!, $status: String){
                perspectiveAddLinkExpression(link: $link, uuid: $uuid, status: $status) {
                    ${LINK_EXPRESSION_FIELDS}
                }
            }`,
            variables: { uuid, link }
        }));
        return perspectiveAddLinkExpression;
    }
    async updateLink(uuid, oldLink, newLink) {
        delete oldLink.__typename;
        delete oldLink.data.__typename;
        delete oldLink.proof.__typename;
        const { perspectiveUpdateLink } = unwrapApolloResult(await __classPrivateFieldGet$8(this, _PerspectiveClient_apolloClient, "f").mutate({
            mutation: gql `mutation perspectiveUpdateLink(
                $uuid: String!,
                $newLink: LinkInput!
                $oldLink: LinkExpressionInput!
            ){
                perspectiveUpdateLink(
                    newLink: $newLink,
                    oldLink: $oldLink,
                    uuid: $uuid
                ) {
                    ${LINK_EXPRESSION_FIELDS}
                }
            }`,
            variables: { uuid, oldLink, newLink }
        }));
        if (!perspectiveUpdateLink.status) {
            delete perspectiveUpdateLink.status;
        }
        return perspectiveUpdateLink;
    }
    async removeLink(uuid, link) {
        delete link.__typename;
        delete link.data.__typename;
        delete link.proof.__typename;
        delete link.status;
        return unwrapApolloResult(await __classPrivateFieldGet$8(this, _PerspectiveClient_apolloClient, "f").mutate({
            mutation: gql `mutation perspectiveRemoveLink($link: LinkExpressionInput!, $uuid: String!) {
                perspectiveRemoveLink(link: $link, uuid: $uuid)
            }`,
            variables: { uuid, link }
        }));
    }
    // ExpressionClient functions, needed for Subjects:
    async getExpression(expressionURI) {
        return await __classPrivateFieldGet$8(this, _PerspectiveClient_expressionClient, "f").get(expressionURI);
    }
    async createExpression(content, languageAddress) {
        return await __classPrivateFieldGet$8(this, _PerspectiveClient_expressionClient, "f").create(content, languageAddress);
    }
    // Subscriptions:
    addPerspectiveAddedListener(cb) {
        __classPrivateFieldGet$8(this, _PerspectiveClient_perspectiveAddedCallbacks, "f").push(cb);
    }
    subscribePerspectiveAdded() {
        __classPrivateFieldGet$8(this, _PerspectiveClient_apolloClient, "f").subscribe({
            query: gql ` subscription {
                perspectiveAdded { ${PERSPECTIVE_HANDLE_FIELDS} }
            }
        `
        }).subscribe({
            next: result => {
                __classPrivateFieldGet$8(this, _PerspectiveClient_perspectiveAddedCallbacks, "f").forEach(cb => {
                    cb(result.data.perspectiveAdded);
                });
            },
            error: (e) => console.error(e)
        });
    }
    addPerspectiveUpdatedListener(cb) {
        __classPrivateFieldGet$8(this, _PerspectiveClient_perspectiveUpdatedCallbacks, "f").push(cb);
    }
    subscribePerspectiveUpdated() {
        __classPrivateFieldGet$8(this, _PerspectiveClient_apolloClient, "f").subscribe({
            query: gql ` subscription {
                perspectiveUpdated { ${PERSPECTIVE_HANDLE_FIELDS} }
            }
        `
        }).subscribe({
            next: result => {
                __classPrivateFieldGet$8(this, _PerspectiveClient_perspectiveUpdatedCallbacks, "f").forEach(cb => {
                    cb(result.data.perspectiveUpdated);
                });
            },
            error: (e) => console.error(e)
        });
    }
    addPerspectiveSyncedListener(cb) {
        __classPrivateFieldGet$8(this, _PerspectiveClient_perspectiveSyncStateChangeCallbacks, "f").push(cb);
    }
    async addPerspectiveSyncStateChangeListener(uuid, cb) {
        __classPrivateFieldGet$8(this, _PerspectiveClient_apolloClient, "f").subscribe({
            query: gql ` subscription {
                perspectiveSyncStateChange(uuid: "${uuid}")
            }
        `
        }).subscribe({
            next: result => {
                cb.forEach(c => {
                    c(result.data.perspectiveSyncStateChange);
                });
            },
            error: (e) => console.error(e)
        });
        await new Promise(resolve => setTimeout(resolve, 500));
    }
    addPerspectiveRemovedListener(cb) {
        __classPrivateFieldGet$8(this, _PerspectiveClient_perspectiveRemovedCallbacks, "f").push(cb);
    }
    subscribePerspectiveRemoved() {
        __classPrivateFieldGet$8(this, _PerspectiveClient_apolloClient, "f").subscribe({
            query: gql ` subscription {
                perspectiveRemoved
            }
        `
        }).subscribe({
            next: result => {
                __classPrivateFieldGet$8(this, _PerspectiveClient_perspectiveRemovedCallbacks, "f").forEach(cb => {
                    cb(result.data.perspectiveRemoved);
                });
            },
            error: (e) => console.error(e)
        });
    }
    async addPerspectiveLinkAddedListener(uuid, cb) {
        __classPrivateFieldGet$8(this, _PerspectiveClient_apolloClient, "f").subscribe({
            query: gql ` subscription {
                perspectiveLinkAdded(uuid: "${uuid}") { ${LINK_EXPRESSION_FIELDS} }
            }
        `
        }).subscribe({
            next: result => {
                cb.forEach(c => {
                    c(result.data.perspectiveLinkAdded);
                });
            },
            error: (e) => console.error(e)
        });
        await new Promise(resolve => setTimeout(resolve, 500));
    }
    async addPerspectiveLinkRemovedListener(uuid, cb) {
        __classPrivateFieldGet$8(this, _PerspectiveClient_apolloClient, "f").subscribe({
            query: gql ` subscription {
                perspectiveLinkRemoved(uuid: "${uuid}") { ${LINK_EXPRESSION_FIELDS} }
            }
        `
        }).subscribe({
            next: result => {
                cb.forEach(c => {
                    if (!result.data.perspectiveLinkRemoved.status) {
                        delete result.data.perspectiveLinkRemoved.status;
                    }
                    c(result.data.perspectiveLinkRemoved);
                });
            },
            error: (e) => console.error(e)
        });
        await new Promise(resolve => setTimeout(resolve, 500));
    }
    async addPerspectiveLinkUpdatedListener(uuid, cb) {
        __classPrivateFieldGet$8(this, _PerspectiveClient_apolloClient, "f").subscribe({
            query: gql ` subscription {
                perspectiveLinkUpdated(uuid: "${uuid}") {
                    oldLink {
                        ${LINK_EXPRESSION_FIELDS}
                    }
                    newLink {
                        ${LINK_EXPRESSION_FIELDS}
                    }
                }
            }
        `
        }).subscribe({
            next: result => {
                cb.forEach(c => {
                    if (!result.data.perspectiveLinkUpdated.newLink.status) {
                        delete result.data.perspectiveLinkUpdated.newLink.status;
                    }
                    if (!result.data.perspectiveLinkUpdated.oldLink.status) {
                        delete result.data.perspectiveLinkUpdated.oldLink.status;
                    }
                    c(result.data.perspectiveLinkUpdated);
                });
            },
            error: (e) => console.error(e)
        });
        await new Promise(resolve => setTimeout(resolve, 500));
    }
    getNeighbourhoodProxy(uuid) {
        return new NeighbourhoodProxy(__classPrivateFieldGet$8(this, _PerspectiveClient_neighbourhoodClient, "f"), uuid);
    }
}
_PerspectiveClient_apolloClient = new WeakMap(), _PerspectiveClient_perspectiveAddedCallbacks = new WeakMap(), _PerspectiveClient_perspectiveUpdatedCallbacks = new WeakMap(), _PerspectiveClient_perspectiveRemovedCallbacks = new WeakMap(), _PerspectiveClient_perspectiveSyncStateChangeCallbacks = new WeakMap(), _PerspectiveClient_expressionClient = new WeakMap(), _PerspectiveClient_neighbourhoodClient = new WeakMap();

var __classPrivateFieldSet$7 = function (receiver, state, value, kind, f) {
    if (kind === "m") throw new TypeError("Private method is not writable");
    if (kind === "a" && !f) throw new TypeError("Private accessor was defined without a setter");
    if (typeof state === "function" ? receiver !== state || !f : !state.has(receiver)) throw new TypeError("Cannot write private member to an object whose class did not declare it");
    return (kind === "a" ? f.call(receiver, value) : f ? f.value = value : state.set(receiver, value)), value;
};
var __classPrivateFieldGet$7 = function (receiver, state, kind, f) {
    if (kind === "a" && !f) throw new TypeError("Private accessor was defined without a getter");
    if (typeof state === "function" ? receiver !== state || !f : !state.has(receiver)) throw new TypeError("Cannot read private member from an object whose class did not declare it");
    return kind === "m" ? f : kind === "a" ? f.call(receiver) : f ? f.value : state.get(receiver);
};
var _AgentClient_apolloClient, _AgentClient_appsChangedCallback, _AgentClient_updatedCallbacks, _AgentClient_agentStatusChangedCallbacks;
const AGENT_SUBITEMS = `
    did
    directMessageLanguage
    perspective { 
        links {
            author, timestamp, 
            proof {
                signature, key, valid, invalid
            }
            data {
                source, predicate, target
            }
        }
    }
`;
const Apps_FIELDS = `
    requestId
    revoked
    auth {
        appName
        appDesc
        appUrl
        appIconPath
        capabilities {
            with {
                domain
                pointers
            }
            can 
        }
    }
`;
const AGENT_STATUS_FIELDS = `
    isInitialized
    isUnlocked
    did
    didDocument
    error
`;
const ENTANGLEMENT_PROOF_FIELDS = `
    did
    didSigningKeyId
    deviceKeyType
    deviceKey
    deviceKeySignedByDid
    didSignedByDeviceKey
`;
const AGENT_SIGNATURE_FIELDS = `
    signature
    publicKey
`;
/**
 * Provides access to all functions regarding the local agent,
 * such as generating, locking, unlocking, importing the DID keystore,
 * as well as updating the publicly shared Agent expression.
 */
class AgentClient {
    constructor(client, subscribe = true) {
        _AgentClient_apolloClient.set(this, void 0);
        _AgentClient_appsChangedCallback.set(this, void 0);
        _AgentClient_updatedCallbacks.set(this, void 0);
        _AgentClient_agentStatusChangedCallbacks.set(this, void 0);
        __classPrivateFieldSet$7(this, _AgentClient_apolloClient, client, "f");
        __classPrivateFieldSet$7(this, _AgentClient_updatedCallbacks, [], "f");
        __classPrivateFieldSet$7(this, _AgentClient_agentStatusChangedCallbacks, [], "f");
        __classPrivateFieldSet$7(this, _AgentClient_appsChangedCallback, [], "f");
        if (subscribe) {
            this.subscribeAgentUpdated();
            this.subscribeAgentStatusChanged();
            this.subscribeAppsChanged();
        }
    }
    /**
     * Returns the Agent expression of the local agent as it is shared
     * publicly via the AgentLanguage.
     *
     * I.e. this is the users profile.
     */
    async me() {
        const { agent } = unwrapApolloResult(await __classPrivateFieldGet$7(this, _AgentClient_apolloClient, "f").query({
            query: gql `query agent { agent { ${AGENT_SUBITEMS} } }`,
        }));
        let agentObject = new Agent(agent.did, agent.perspective);
        agentObject.directMessageLanguage = agent.directMessageLanguage;
        return agentObject;
    }
    async status() {
        const { agentStatus } = unwrapApolloResult(await __classPrivateFieldGet$7(this, _AgentClient_apolloClient, "f").query({
            query: gql `query agentStatus {
                agentStatus {
                    ${AGENT_STATUS_FIELDS}
                }
            }`,
        }));
        return new AgentStatus(agentStatus);
    }
    async generate(passphrase) {
        const { agentGenerate } = unwrapApolloResult(await __classPrivateFieldGet$7(this, _AgentClient_apolloClient, "f").mutate({
            mutation: gql `mutation agentGenerate(
                $passphrase: String!
            ) {
                agentGenerate(passphrase: $passphrase) {
                    ${AGENT_STATUS_FIELDS}
                }
            }`,
            variables: { passphrase },
        }));
        return new AgentStatus(agentGenerate);
    }
    async import(args) {
        let { did, didDocument, keystore, passphrase } = args;
        const { agentImport } = unwrapApolloResult(await __classPrivateFieldGet$7(this, _AgentClient_apolloClient, "f").mutate({
            mutation: gql `mutation agentImport(
                $did: String!,
                $didDocument: String!,
                $keystore: String!,
                $passphrase: String!
            ) {
                agentImport(did: $did, didDocument: $didDocument, keystore: $keystore, passphrase: $passphrase) {
                    ${AGENT_STATUS_FIELDS}
                }
            }`,
            variables: { did, didDocument, keystore, passphrase },
        }));
        return new AgentStatus(agentImport);
    }
    async lock(passphrase) {
        const { agentLock } = unwrapApolloResult(await __classPrivateFieldGet$7(this, _AgentClient_apolloClient, "f").mutate({
            mutation: gql `mutation agentLock($passphrase: String!) {
                agentLock(passphrase: $passphrase) {
                    ${AGENT_STATUS_FIELDS}
                }
            }`,
            variables: { passphrase },
        }));
        return new AgentStatus(agentLock);
    }
    async unlock(passphrase) {
        const { agentUnlock } = unwrapApolloResult(await __classPrivateFieldGet$7(this, _AgentClient_apolloClient, "f").mutate({
            mutation: gql `mutation agentUnlock($passphrase: String!) {
                agentUnlock(passphrase: $passphrase) {
                    ${AGENT_STATUS_FIELDS}
                }
            }`,
            variables: { passphrase },
        }));
        return new AgentStatus(agentUnlock);
    }
    async byDID(did) {
        const { agentByDID } = unwrapApolloResult(await __classPrivateFieldGet$7(this, _AgentClient_apolloClient, "f").query({
            query: gql `query agentByDID($did: String!) {
                agentByDID(did: $did) {
                    ${AGENT_SUBITEMS}
                }
            }`,
            variables: { did },
        }));
        return agentByDID;
    }
    async updatePublicPerspective(perspective) {
        const cleanedPerspective = JSON.parse(JSON.stringify(perspective));
        delete cleanedPerspective.__typename;
        cleanedPerspective.links.forEach((link) => {
            delete link.__typename;
            delete link.data.__typename;
            delete link.proof.__typename;
            delete link.status;
        });
        const { agentUpdatePublicPerspective } = unwrapApolloResult(await __classPrivateFieldGet$7(this, _AgentClient_apolloClient, "f").mutate({
            mutation: gql `mutation agentUpdatePublicPerspective($perspective: PerspectiveInput!) {
                agentUpdatePublicPerspective(perspective: $perspective) {
                    ${AGENT_SUBITEMS}
                }
            }`,
            variables: { perspective: cleanedPerspective },
        }));
        const a = agentUpdatePublicPerspective;
        const agent = new Agent(a.did, a.perspective);
        agent.directMessageLanguage = a.directMessageLanguage;
        return agent;
    }
    async mutatePublicPerspective(mutations) {
        const perspectiveClient = new PerspectiveClient(__classPrivateFieldGet$7(this, _AgentClient_apolloClient, "f"));
        const agentClient = new AgentClient(__classPrivateFieldGet$7(this, _AgentClient_apolloClient, "f"));
        //Create the proxy perspective and load existing links
        const proxyPerspective = await perspectiveClient.add("Agent Perspective Proxy");
        const agentMe = await agentClient.me();
        if (agentMe.perspective) {
            await proxyPerspective.loadSnapshot(agentMe.perspective);
        }
        //Make the mutations on the proxy perspective
        for (const addition of mutations.additions) {
            await proxyPerspective.add(addition);
        }
        for (const removal of mutations.removals) {
            await proxyPerspective.remove(removal);
        }
        //Get the snapshot of the proxy perspective
        const snapshot = await proxyPerspective.snapshot();
        //Update the users public perspective
        const agent = await this.updatePublicPerspective(snapshot);
        //Cleanup and return
        await perspectiveClient.remove(proxyPerspective.uuid);
        return agent;
    }
    async updateDirectMessageLanguage(directMessageLanguage) {
        const { agentUpdateDirectMessageLanguage } = unwrapApolloResult(await __classPrivateFieldGet$7(this, _AgentClient_apolloClient, "f").mutate({
            mutation: gql `mutation agentUpdateDirectMessageLanguage($directMessageLanguage: String!) {
                agentUpdateDirectMessageLanguage(directMessageLanguage: $directMessageLanguage) {
                    ${AGENT_SUBITEMS}
                }
            }`,
            variables: { directMessageLanguage },
        }));
        const a = agentUpdateDirectMessageLanguage;
        const agent = new Agent(a.did, a.perspective);
        agent.directMessageLanguage = a.directMessageLanguage;
        return agent;
    }
    async addEntanglementProofs(proofs) {
        const { agentAddEntanglementProofs } = unwrapApolloResult(await __classPrivateFieldGet$7(this, _AgentClient_apolloClient, "f").mutate({
            mutation: gql `mutation agentAddEntanglementProofs($proofs: [EntanglementProofInput!]!) {
                agentAddEntanglementProofs(proofs: $proofs) {
                    ${ENTANGLEMENT_PROOF_FIELDS}
                }
            }`,
            variables: { proofs },
        }));
        return agentAddEntanglementProofs;
    }
    async deleteEntanglementProofs(proofs) {
        const { agentDeleteEntanglementProofs } = unwrapApolloResult(await __classPrivateFieldGet$7(this, _AgentClient_apolloClient, "f").mutate({
            mutation: gql `mutation agentDeleteEntanglementProofs($proofs: [EntanglementProofInput!]!) {
                agentDeleteEntanglementProofs(proofs: $proofs) {
                    ${ENTANGLEMENT_PROOF_FIELDS}
                }
            }`,
            variables: { proofs },
        }));
        return agentDeleteEntanglementProofs;
    }
    async getEntanglementProofs() {
        const { agentGetEntanglementProofs } = unwrapApolloResult(await __classPrivateFieldGet$7(this, _AgentClient_apolloClient, "f").query({
            query: gql `query agentGetEntanglementProofs {
                agentGetEntanglementProofs {
                    ${ENTANGLEMENT_PROOF_FIELDS}
                }
            }`,
        }));
        return agentGetEntanglementProofs;
    }
    async entanglementProofPreFlight(deviceKey, deviceKeyType) {
        const { agentEntanglementProofPreFlight } = unwrapApolloResult(await __classPrivateFieldGet$7(this, _AgentClient_apolloClient, "f").mutate({
            mutation: gql `mutation agentEntanglementProofPreFlight($deviceKey: String!, $deviceKeyType: String!) {
                agentEntanglementProofPreFlight(deviceKey: $deviceKey, deviceKeyType: $deviceKeyType) {
                    ${ENTANGLEMENT_PROOF_FIELDS}
                }
            }`,
            variables: { deviceKey, deviceKeyType },
        }));
        return agentEntanglementProofPreFlight;
    }
    addUpdatedListener(listener) {
        __classPrivateFieldGet$7(this, _AgentClient_updatedCallbacks, "f").push(listener);
    }
    addAppChangedListener(listener) {
        __classPrivateFieldGet$7(this, _AgentClient_appsChangedCallback, "f").push(listener);
    }
    subscribeAgentUpdated() {
        __classPrivateFieldGet$7(this, _AgentClient_apolloClient, "f")
            .subscribe({
            query: gql ` subscription {
                agentUpdated { ${AGENT_SUBITEMS} }
            }   
        `,
        })
            .subscribe({
            next: (result) => {
                const agent = result.data.agentUpdated;
                __classPrivateFieldGet$7(this, _AgentClient_updatedCallbacks, "f").forEach((cb) => {
                    cb(agent);
                });
            },
            error: (e) => console.error(e),
        });
    }
    subscribeAppsChanged() {
        __classPrivateFieldGet$7(this, _AgentClient_apolloClient, "f")
            .subscribe({
            query: gql ` subscription {
                agentAppsChanged { 
                  ${Apps_FIELDS}
                }
            }   
        `,
        })
            .subscribe({
            next: (result) => {
                __classPrivateFieldGet$7(this, _AgentClient_appsChangedCallback, "f").forEach((cb) => {
                    cb();
                });
            },
            error: (e) => console.error(e),
        });
    }
    addAgentStatusChangedListener(listener) {
        __classPrivateFieldGet$7(this, _AgentClient_agentStatusChangedCallbacks, "f").push(listener);
    }
    subscribeAgentStatusChanged() {
        __classPrivateFieldGet$7(this, _AgentClient_apolloClient, "f")
            .subscribe({
            query: gql ` subscription {
                agentStatusChanged { ${AGENT_STATUS_FIELDS} }
            }   
        `,
        })
            .subscribe({
            next: (result) => {
                const agent = result.data.agentStatusChanged;
                __classPrivateFieldGet$7(this, _AgentClient_agentStatusChangedCallbacks, "f").forEach((cb) => {
                    cb(agent);
                });
            },
            error: (e) => console.error(e),
        });
    }
    async requestCapability(authInfo) {
        const { agentRequestCapability } = unwrapApolloResult(await __classPrivateFieldGet$7(this, _AgentClient_apolloClient, "f").mutate({
            mutation: gql `
          mutation agentRequestCapability($authInfo: AuthInfoInput!) {
            agentRequestCapability(authInfo: $authInfo)
          }
        `,
            variables: { authInfo },
        }));
        return agentRequestCapability;
    }
    async permitCapability(auth) {
        const { agentPermitCapability } = unwrapApolloResult(await __classPrivateFieldGet$7(this, _AgentClient_apolloClient, "f").mutate({
            mutation: gql `
          mutation agentPermitCapability($auth: String!) {
            agentPermitCapability(auth: $auth)
          }
        `,
            variables: { auth },
        }));
        return agentPermitCapability;
    }
    async generateJwt(requestId, rand) {
        const { agentGenerateJwt } = unwrapApolloResult(await __classPrivateFieldGet$7(this, _AgentClient_apolloClient, "f").mutate({
            mutation: gql `
          mutation agentGenerateJwt($requestId: String!, $rand: String!) {
            agentGenerateJwt(requestId: $requestId, rand: $rand)
          }
        `,
            variables: { requestId, rand },
        }));
        return agentGenerateJwt;
    }
    async getApps() {
        const { agentGetApps } = unwrapApolloResult(await __classPrivateFieldGet$7(this, _AgentClient_apolloClient, "f").mutate({
            mutation: gql `query agentGetApps {
                agentGetApps {
                    ${Apps_FIELDS}
                }
            }`,
        }));
        return agentGetApps;
    }
    async removeApp(requestId) {
        const { agentRemoveApp } = unwrapApolloResult(await __classPrivateFieldGet$7(this, _AgentClient_apolloClient, "f").mutate({
            mutation: gql `mutation agentRemoveApp($requestId: String!) {
                agentRemoveApp(requestId: $requestId) {
                    ${Apps_FIELDS}
                }
            }`,
            variables: { requestId },
        }));
        return agentRemoveApp;
    }
    async revokeToken(requestId) {
        const { agentRevokeToken } = unwrapApolloResult(await __classPrivateFieldGet$7(this, _AgentClient_apolloClient, "f").mutate({
            mutation: gql `mutation agentRevokeToken($requestId: String!) {
                agentRevokeToken(requestId: $requestId) {
                    ${Apps_FIELDS}
                }
            }`,
            variables: { requestId },
        }));
        return agentRevokeToken;
    }
    async isLocked() {
        const { agentIsLocked } = unwrapApolloResult(await __classPrivateFieldGet$7(this, _AgentClient_apolloClient, "f").mutate({
            mutation: gql `
          query agentIsLocked {
            agentIsLocked
          }
        `,
        }));
        return agentIsLocked;
    }
    async signMessage(message) {
        const { agentSignMessage } = unwrapApolloResult(await __classPrivateFieldGet$7(this, _AgentClient_apolloClient, "f").mutate({
            mutation: gql `mutation agentSignMessage($message: String!) {
          agentSignMessage(message: $message) {
              ${AGENT_SIGNATURE_FIELDS}
          }
        }`,
            variables: { message },
        }));
        return agentSignMessage;
    }
}
_AgentClient_apolloClient = new WeakMap(), _AgentClient_appsChangedCallback = new WeakMap(), _AgentClient_updatedCallbacks = new WeakMap(), _AgentClient_agentStatusChangedCallbacks = new WeakMap();

var __classPrivateFieldSet$6 = function (receiver, state, value, kind, f) {
    if (kind === "m") throw new TypeError("Private method is not writable");
    if (kind === "a" && !f) throw new TypeError("Private accessor was defined without a setter");
    if (typeof state === "function" ? receiver !== state || !f : !state.has(receiver)) throw new TypeError("Cannot write private member to an object whose class did not declare it");
    return (kind === "a" ? f.call(receiver, value) : f ? f.value = value : state.set(receiver, value)), value;
};
var __classPrivateFieldGet$6 = function (receiver, state, kind, f) {
    if (kind === "a" && !f) throw new TypeError("Private accessor was defined without a getter");
    if (typeof state === "function" ? receiver !== state || !f : !state.has(receiver)) throw new TypeError("Cannot read private member from an object whose class did not declare it");
    return kind === "m" ? f : kind === "a" ? f.call(receiver) : f ? f.value : state.get(receiver);
};
var _LanguageClient_apolloClient;
const LANGUAGE_COMPLETE = `
    name
    address
    settings
    icon { code }
    constructorIcon { code }
    settingsIcon { code }
`;
const LANGUAGE_META = `
    name
    address
    description
    author
    templated
    templateSourceLanguageAddress
    templateAppliedParams
    possibleTemplateParams
    sourceCodeLink
`;
class LanguageClient {
    constructor(apolloClient) {
        _LanguageClient_apolloClient.set(this, void 0);
        __classPrivateFieldSet$6(this, _LanguageClient_apolloClient, apolloClient, "f");
    }
    async byAddress(address) {
        const { language } = unwrapApolloResult(await __classPrivateFieldGet$6(this, _LanguageClient_apolloClient, "f").query({
            query: gql `query byAddress($address: String!) {
                language(address: $address) {
                    ${LANGUAGE_COMPLETE}
                }
            }`,
            variables: { address }
        }));
        return language;
    }
    async byFilter(filter) {
        const { languages } = unwrapApolloResult(await __classPrivateFieldGet$6(this, _LanguageClient_apolloClient, "f").query({
            query: gql `query byFilter($filter: String!) {
                languages(filter: $filter) {
                    ${LANGUAGE_COMPLETE}
                }
            }`,
            variables: { filter }
        }));
        return languages;
    }
    async all() {
        return this.byFilter('');
    }
    async writeSettings(languageAddress, settings) {
        const { languageWriteSettings } = unwrapApolloResult(await __classPrivateFieldGet$6(this, _LanguageClient_apolloClient, "f").mutate({
            mutation: gql `mutation writeSettings($languageAddress: String!, $settings: String!) {
                languageWriteSettings(languageAddress: $languageAddress, settings: $settings)
            }`,
            variables: { languageAddress, settings }
        }));
        return languageWriteSettings;
    }
    async applyTemplateAndPublish(sourceLanguageHash, templateData) {
        const { languageApplyTemplateAndPublish } = unwrapApolloResult(await __classPrivateFieldGet$6(this, _LanguageClient_apolloClient, "f").mutate({
            mutation: gql `mutation languageApplyTemplateAndPublish(
                $sourceLanguageHash: String!,
                $templateData: String!,
            ) {
                languageApplyTemplateAndPublish(sourceLanguageHash: $sourceLanguageHash, templateData: $templateData) {
                    name, address
                }
            }`,
            variables: { sourceLanguageHash, templateData }
        }));
        return languageApplyTemplateAndPublish;
    }
    async publish(languagePath, languageMeta) {
        const { languagePublish } = unwrapApolloResult(await __classPrivateFieldGet$6(this, _LanguageClient_apolloClient, "f").mutate({
            mutation: gql `mutation languagePublish(
                $languagePath: String!,
                $languageMeta: LanguageMetaInput!,
            ) {
                languagePublish(languagePath: $languagePath, languageMeta: $languageMeta) {
                    ${LANGUAGE_META}
                }
            }`,
            variables: { languagePath, languageMeta }
        }));
        return languagePublish;
    }
    async meta(address) {
        const { languageMeta } = unwrapApolloResult(await __classPrivateFieldGet$6(this, _LanguageClient_apolloClient, "f").query({
            query: gql `query languageMeta(
                $address: String!,
            ) {
                languageMeta(address: $address) {
                    ${LANGUAGE_META}
                }
            }`,
            variables: { address }
        }));
        return languageMeta;
    }
    async source(address) {
        const { languageSource } = unwrapApolloResult(await __classPrivateFieldGet$6(this, _LanguageClient_apolloClient, "f").query({
            query: gql `query languageSource(
                $address: String!,
            ) {
                languageSource(address: $address)
            }`,
            variables: { address }
        }));
        return languageSource;
    }
    async remove(address) {
        const { languageRemove } = unwrapApolloResult(await __classPrivateFieldGet$6(this, _LanguageClient_apolloClient, "f").mutate({
            mutation: gql `mutation languageRemove(
                $address: String!,
            ) {
                languageRemove(address: $address)
            }`,
            variables: { address }
        }));
        return languageRemove;
    }
}
_LanguageClient_apolloClient = new WeakMap();

var __classPrivateFieldSet$5 = function (receiver, state, value, kind, f) {
    if (kind === "m") throw new TypeError("Private method is not writable");
    if (kind === "a" && !f) throw new TypeError("Private accessor was defined without a setter");
    if (typeof state === "function" ? receiver !== state || !f : !state.has(receiver)) throw new TypeError("Cannot write private member to an object whose class did not declare it");
    return (kind === "a" ? f.call(receiver, value) : f ? f.value = value : state.set(receiver, value)), value;
};
var __classPrivateFieldGet$5 = function (receiver, state, kind, f) {
    if (kind === "a" && !f) throw new TypeError("Private accessor was defined without a getter");
    if (typeof state === "function" ? receiver !== state || !f : !state.has(receiver)) throw new TypeError("Cannot read private member from an object whose class did not declare it");
    return kind === "m" ? f : kind === "a" ? f.call(receiver) : f ? f.value : state.get(receiver);
};
var _NeighbourhoodClient_apolloClient, _NeighbourhoodClient_signalHandlers;
class NeighbourhoodClient {
    constructor(client) {
        _NeighbourhoodClient_apolloClient.set(this, void 0);
        _NeighbourhoodClient_signalHandlers.set(this, new Map());
        __classPrivateFieldSet$5(this, _NeighbourhoodClient_apolloClient, client, "f");
    }
    async publishFromPerspective(perspectiveUUID, linkLanguage, meta) {
        const { neighbourhoodPublishFromPerspective } = unwrapApolloResult(await __classPrivateFieldGet$5(this, _NeighbourhoodClient_apolloClient, "f").mutate({
            mutation: gql `mutation neighbourhoodPublishFromPerspective(
                $linkLanguage: String!,
                $meta: PerspectiveInput!,
                $perspectiveUUID: String!
            ) {
                neighbourhoodPublishFromPerspective(
                    linkLanguage: $linkLanguage,
                    meta: $meta,
                    perspectiveUUID: $perspectiveUUID
                )
            }`,
            variables: { perspectiveUUID, linkLanguage, meta: meta }
        }));
        return neighbourhoodPublishFromPerspective;
    }
    async joinFromUrl(url) {
        const { neighbourhoodJoinFromUrl } = unwrapApolloResult(await __classPrivateFieldGet$5(this, _NeighbourhoodClient_apolloClient, "f").mutate({
            mutation: gql `mutation neighbourhoodJoinFromUrl($url: String!) {
                neighbourhoodJoinFromUrl(url: $url) {
                    uuid
                    name
                    sharedUrl
                    state
                    neighbourhood { 
                        data {
                            linkLanguage 
                            meta { 
                                links
                                    {
                                        author
                                        timestamp
                                        data { source, predicate, target }
                                        proof { valid, invalid, signature, key }
                                    }  
                            } 
                        }
                        author
                    }
                }
            }`,
            variables: { url }
        }));
        return neighbourhoodJoinFromUrl;
    }
    async otherAgents(perspectiveUUID) {
        const { neighbourhoodOtherAgents } = unwrapApolloResult(await __classPrivateFieldGet$5(this, _NeighbourhoodClient_apolloClient, "f").query({
            query: gql `query neighbourhoodOtherAgents($perspectiveUUID: String!) {
                neighbourhoodOtherAgents(perspectiveUUID: $perspectiveUUID)
            }`,
            variables: { perspectiveUUID }
        }));
        return neighbourhoodOtherAgents;
    }
    async hasTelepresenceAdapter(perspectiveUUID) {
        const { neighbourhoodHasTelepresenceAdapter } = unwrapApolloResult(await __classPrivateFieldGet$5(this, _NeighbourhoodClient_apolloClient, "f").query({
            query: gql `query neighbourhoodHasTelepresenceAdapter($perspectiveUUID: String!) {
                neighbourhoodHasTelepresenceAdapter(perspectiveUUID: $perspectiveUUID)
            }`,
            variables: { perspectiveUUID }
        }));
        return neighbourhoodHasTelepresenceAdapter;
    }
    async onlineAgents(perspectiveUUID) {
        const { neighbourhoodOnlineAgents } = unwrapApolloResult(await __classPrivateFieldGet$5(this, _NeighbourhoodClient_apolloClient, "f").query({
            query: gql `query neighbourhoodOnlineAgents($perspectiveUUID: String!) {
                neighbourhoodOnlineAgents(perspectiveUUID: $perspectiveUUID) {
                    did
                    status {
                        author
                        timestamp
                        data { 
                            links {
                                author
                                timestamp
                                data { source, predicate, target }
                                proof { valid, invalid, signature, key }
                            }
                        }
                        proof { valid, invalid, signature, key }
                    }
                }
            }`,
            variables: { perspectiveUUID }
        }));
        return neighbourhoodOnlineAgents;
    }
    async setOnlineStatus(perspectiveUUID, status) {
        const { neighbourhoodSetOnlineStatus } = unwrapApolloResult(await __classPrivateFieldGet$5(this, _NeighbourhoodClient_apolloClient, "f").mutate({
            mutation: gql `mutation neighbourhoodSetOnlineStatus(
                $perspectiveUUID: String!,
                $status: PerspectiveInput!
            ) {
                neighbourhoodSetOnlineStatus(
                    perspectiveUUID: $perspectiveUUID,
                    status: $status
                )
            }`,
            variables: { perspectiveUUID, status }
        }));
        return neighbourhoodSetOnlineStatus;
    }
    async setOnlineStatusU(perspectiveUUID, status) {
        const { neighbourhoodSetOnlineStatusU } = unwrapApolloResult(await __classPrivateFieldGet$5(this, _NeighbourhoodClient_apolloClient, "f").mutate({
            mutation: gql `mutation neighbourhoodSetOnlineStatusU(
                $perspectiveUUID: String!,
                $status: PerspectiveUnsignedInput!
            ) {
                neighbourhoodSetOnlineStatusU(
                    perspectiveUUID: $perspectiveUUID,
                    status: $status
                )
            }`,
            variables: { perspectiveUUID, status }
        }));
        return neighbourhoodSetOnlineStatusU;
    }
    async sendSignal(perspectiveUUID, remoteAgentDid, payload) {
        const { neighbourhoodSendSignal } = unwrapApolloResult(await __classPrivateFieldGet$5(this, _NeighbourhoodClient_apolloClient, "f").mutate({
            mutation: gql `mutation neighbourhoodSendSignal(
                $perspectiveUUID: String!,
                $remoteAgentDid: String!,
                $payload: PerspectiveInput!
            ) {
                neighbourhoodSendSignal(
                    perspectiveUUID: $perspectiveUUID,
                    remoteAgentDid: $remoteAgentDid,
                    payload: $payload
                )
            }`,
            variables: { perspectiveUUID, remoteAgentDid, payload }
        }));
        return neighbourhoodSendSignal;
    }
    async sendSignalU(perspectiveUUID, remoteAgentDid, payload) {
        const { neighbourhoodSendSignalU } = unwrapApolloResult(await __classPrivateFieldGet$5(this, _NeighbourhoodClient_apolloClient, "f").mutate({
            mutation: gql `mutation neighbourhoodSendSignalU(
                $perspectiveUUID: String!,
                $remoteAgentDid: String!,
                $payload: PerspectiveUnsignedInput!
            ) {
                neighbourhoodSendSignalU(
                    perspectiveUUID: $perspectiveUUID,
                    remoteAgentDid: $remoteAgentDid,
                    payload: $payload
                )
            }`,
            variables: { perspectiveUUID, remoteAgentDid, payload }
        }));
        return neighbourhoodSendSignalU;
    }
    async sendBroadcast(perspectiveUUID, payload) {
        const { neighbourhoodSendBroadcast } = unwrapApolloResult(await __classPrivateFieldGet$5(this, _NeighbourhoodClient_apolloClient, "f").mutate({
            mutation: gql `mutation neighbourhoodSendBroadcast(
                $perspectiveUUID: String!,
                $payload: PerspectiveInput!
            ) {
                neighbourhoodSendBroadcast(
                    perspectiveUUID: $perspectiveUUID,
                    payload: $payload
                )
            }`,
            variables: { perspectiveUUID, payload }
        }));
        return neighbourhoodSendBroadcast;
    }
    async sendBroadcastU(perspectiveUUID, payload) {
        const { neighbourhoodSendBroadcastU } = unwrapApolloResult(await __classPrivateFieldGet$5(this, _NeighbourhoodClient_apolloClient, "f").mutate({
            mutation: gql `mutation neighbourhoodSendBroadcastU(
                $perspectiveUUID: String!,
                $payload: PerspectiveUnsignedInput!
            ) {
                neighbourhoodSendBroadcastU(
                    perspectiveUUID: $perspectiveUUID,
                    payload: $payload
                )
            }`,
            variables: { perspectiveUUID, payload }
        }));
        return neighbourhoodSendBroadcastU;
    }
    dispatchSignal(perspectiveUUID, signal) {
        const handlers = __classPrivateFieldGet$5(this, _NeighbourhoodClient_signalHandlers, "f").get(perspectiveUUID);
        if (handlers) {
            handlers.forEach(handler => handler(signal));
        }
    }
    async subscribeToSignals(perspectiveUUID) {
        const that = this;
        await __classPrivateFieldGet$5(this, _NeighbourhoodClient_apolloClient, "f").subscribe({
            query: gql `subscription neighbourhoodSignal($perspectiveUUID: String!) {
                neighbourhoodSignal(perspectiveUUID: $perspectiveUUID) {
                    author
                    timestamp
                    data {
                        links
                            {
                                author
                                timestamp
                                data { source, predicate, target }
                                proof { valid, invalid, signature, key }
                            }  
                    }
                    proof { valid, invalid, signature, key }
                }
            }`,
            variables: { perspectiveUUID }
        }).subscribe({
            next: (result) => {
                const { neighbourhoodSignal } = unwrapApolloResult(result);
                that.dispatchSignal(perspectiveUUID, neighbourhoodSignal);
            }
        });
    }
    async addSignalHandler(perspectiveUUID, handler) {
        let handlersForPerspective = __classPrivateFieldGet$5(this, _NeighbourhoodClient_signalHandlers, "f").get(perspectiveUUID);
        if (!handlersForPerspective) {
            handlersForPerspective = [];
            await this.subscribeToSignals(perspectiveUUID);
        }
        handlersForPerspective.push(handler);
        __classPrivateFieldGet$5(this, _NeighbourhoodClient_signalHandlers, "f").set(perspectiveUUID, handlersForPerspective);
    }
    removeSignalHandler(perspectiveUUID, handler) {
        const handlersForPerspective = __classPrivateFieldGet$5(this, _NeighbourhoodClient_signalHandlers, "f").get(perspectiveUUID);
        if (handlersForPerspective) {
            const index = handlersForPerspective.indexOf(handler);
            if (index > -1) {
                handlersForPerspective.splice(index, 1);
            }
        }
        __classPrivateFieldGet$5(this, _NeighbourhoodClient_signalHandlers, "f").set(perspectiveUUID, handlersForPerspective);
    }
}
_NeighbourhoodClient_apolloClient = new WeakMap(), _NeighbourhoodClient_signalHandlers = new WeakMap();

var __classPrivateFieldSet$4 = function (receiver, state, value, kind, f) {
    if (kind === "m") throw new TypeError("Private method is not writable");
    if (kind === "a" && !f) throw new TypeError("Private accessor was defined without a setter");
    if (typeof state === "function" ? receiver !== state || !f : !state.has(receiver)) throw new TypeError("Cannot write private member to an object whose class did not declare it");
    return (kind === "a" ? f.call(receiver, value) : f ? f.value = value : state.set(receiver, value)), value;
};
var __classPrivateFieldGet$4 = function (receiver, state, kind, f) {
    if (kind === "a" && !f) throw new TypeError("Private accessor was defined without a getter");
    if (typeof state === "function" ? receiver !== state || !f : !state.has(receiver)) throw new TypeError("Cannot read private member from an object whose class did not declare it");
    return kind === "m" ? f : kind === "a" ? f.call(receiver) : f ? f.value : state.get(receiver);
};
var _RuntimeClient_apolloClient, _RuntimeClient_messageReceivedCallbacks, _RuntimeClient_exceptionOccurredCallbacks;
const PERSPECTIVE_EXPRESSION_FIELDS = `
author
timestamp
data { 
    links {
        author
        timestamp
        data { source, predicate, target }
        proof { valid, invalid, signature, key }
    }  
}
proof { valid, invalid, signature, key }
`;
class RuntimeClient {
    constructor(client, subscribe = true) {
        _RuntimeClient_apolloClient.set(this, void 0);
        _RuntimeClient_messageReceivedCallbacks.set(this, void 0);
        _RuntimeClient_exceptionOccurredCallbacks.set(this, void 0);
        __classPrivateFieldSet$4(this, _RuntimeClient_apolloClient, client, "f");
        __classPrivateFieldSet$4(this, _RuntimeClient_messageReceivedCallbacks, [], "f");
        __classPrivateFieldSet$4(this, _RuntimeClient_exceptionOccurredCallbacks, [], "f");
        if (subscribe) {
            this.subscribeMessageReceived();
            this.subscribeExceptionOccurred();
        }
    }
    async info() {
        const { runtimeInfo } = unwrapApolloResult(await __classPrivateFieldGet$4(this, _RuntimeClient_apolloClient, "f").query({
            query: gql `query runtimeInfo {
                runtimeInfo {
                    ad4mExecutorVersion,
                    isInitialized,
                    isUnlocked
                }
            }`,
        }));
        return runtimeInfo;
    }
    async quit() {
        const result = unwrapApolloResult(await __classPrivateFieldGet$4(this, _RuntimeClient_apolloClient, "f").mutate({
            mutation: gql `mutation runtimeQuit { runtimeQuit }`
        }));
        return result.runtimeQuit;
    }
    async openLink(url) {
        const { runtimeOpenLink } = unwrapApolloResult(await __classPrivateFieldGet$4(this, _RuntimeClient_apolloClient, "f").mutate({
            mutation: gql `mutation runtimeOpenLink($url: String!) {
                runtimeOpenLink(url: $url)
            }`,
            variables: { url }
        }));
        return runtimeOpenLink;
    }
    async addTrustedAgents(agents) {
        const { addTrustedAgents } = unwrapApolloResult(await __classPrivateFieldGet$4(this, _RuntimeClient_apolloClient, "f").mutate({
            mutation: gql `mutation addTrustedAgents($agents: [String!]!) {
                addTrustedAgents(agents: $agents)
            }`,
            variables: { agents }
        }));
        return addTrustedAgents;
    }
    async deleteTrustedAgents(agents) {
        const { deleteTrustedAgents } = unwrapApolloResult(await __classPrivateFieldGet$4(this, _RuntimeClient_apolloClient, "f").mutate({
            mutation: gql `mutation deleteTrustedAgents($agents: [String!]!) {
                deleteTrustedAgents(agents: $agents)
            }`,
            variables: { agents }
        }));
        return deleteTrustedAgents;
    }
    async getTrustedAgents() {
        const { getTrustedAgents } = unwrapApolloResult(await __classPrivateFieldGet$4(this, _RuntimeClient_apolloClient, "f").query({
            query: gql `query getTrustedAgents {
                getTrustedAgents
            }`,
        }));
        return getTrustedAgents;
    }
    async addKnownLinkLanguageTemplates(addresses) {
        const { runtimeAddKnownLinkLanguageTemplates } = unwrapApolloResult(await __classPrivateFieldGet$4(this, _RuntimeClient_apolloClient, "f").mutate({
            mutation: gql `mutation runtimeAddKnownLinkLanguageTemplates($addresses: [String!]!) {
                runtimeAddKnownLinkLanguageTemplates(addresses: $addresses)
            }`,
            variables: { addresses }
        }));
        return runtimeAddKnownLinkLanguageTemplates;
    }
    async removeKnownLinkLanguageTemplates(addresses) {
        const { runtimeRemoveKnownLinkLanguageTemplates } = unwrapApolloResult(await __classPrivateFieldGet$4(this, _RuntimeClient_apolloClient, "f").mutate({
            mutation: gql `mutation runtimeRemoveKnownLinkLanguageTemplates($addresses: [String!]!) {
                runtimeRemoveKnownLinkLanguageTemplates(addresses: $addresses)
            }`,
            variables: { addresses }
        }));
        return runtimeRemoveKnownLinkLanguageTemplates;
    }
    async knownLinkLanguageTemplates() {
        const { runtimeKnownLinkLanguageTemplates } = unwrapApolloResult(await __classPrivateFieldGet$4(this, _RuntimeClient_apolloClient, "f").query({
            query: gql `query runtimeKnownLinkLanguageTemplates {
                runtimeKnownLinkLanguageTemplates
            }`,
        }));
        return runtimeKnownLinkLanguageTemplates;
    }
    async addFriends(dids) {
        const { runtimeAddFriends } = unwrapApolloResult(await __classPrivateFieldGet$4(this, _RuntimeClient_apolloClient, "f").mutate({
            mutation: gql `mutation runtimeAddFriends($dids: [String!]!) {
                runtimeAddFriends(dids: $dids)
            }`,
            variables: { dids }
        }));
        return runtimeAddFriends;
    }
    async removeFriends(dids) {
        const { runtimeRemoveFriends } = unwrapApolloResult(await __classPrivateFieldGet$4(this, _RuntimeClient_apolloClient, "f").mutate({
            mutation: gql `mutation runtimeRemoveFriends($dids: [String!]!) {
                runtimeRemoveFriends(dids: $dids)
            }`,
            variables: { dids }
        }));
        return runtimeRemoveFriends;
    }
    async friends() {
        const { runtimeFriends } = unwrapApolloResult(await __classPrivateFieldGet$4(this, _RuntimeClient_apolloClient, "f").query({
            query: gql `query runtimeFriends {
                runtimeFriends
            }`,
        }));
        return runtimeFriends;
    }
    async hcAgentInfos() {
        const { runtimeHcAgentInfos } = unwrapApolloResult(await __classPrivateFieldGet$4(this, _RuntimeClient_apolloClient, "f").query({
            query: gql `query runtimeHcAgentInfos {
                runtimeHcAgentInfos
            }`,
        }));
        return runtimeHcAgentInfos;
    }
    async hcAddAgentInfos(agentInfos) {
        const { runtimeHcAddAgentInfos } = unwrapApolloResult(await __classPrivateFieldGet$4(this, _RuntimeClient_apolloClient, "f").mutate({
            mutation: gql `mutation runtimeHcAddAgentInfos($agentInfos: String!) {
                runtimeHcAddAgentInfos(agentInfos: $agentInfos)
            }`,
            variables: { agentInfos }
        }));
        return runtimeHcAddAgentInfos;
    }
    async verifyStringSignedByDid(did, didSigningKeyId, data, signedData) {
        const { runtimeVerifyStringSignedByDid } = unwrapApolloResult(await __classPrivateFieldGet$4(this, _RuntimeClient_apolloClient, "f").mutate({
            mutation: gql `query runtimeVerifyStringSignedByDid($did: String!, $didSigningKeyId: String!, $data: String!, $signedData: String!) {
                runtimeVerifyStringSignedByDid(did: $did, didSigningKeyId: $didSigningKeyId, data: $data, signedData: $signedData)
            }`,
            variables: { did, didSigningKeyId, data, signedData }
        }));
        return runtimeVerifyStringSignedByDid;
    }
    async setStatus(perspective) {
        const { runtimeSetStatus } = unwrapApolloResult(await __classPrivateFieldGet$4(this, _RuntimeClient_apolloClient, "f").mutate({
            mutation: gql `mutation runtimeSetStatus($status: PerspectiveInput!) {
                runtimeSetStatus(status: $status)
            }`,
            variables: { status: perspective }
        }));
        return runtimeSetStatus;
    }
    async friendStatus(did) {
        const { runtimeFriendStatus } = unwrapApolloResult(await __classPrivateFieldGet$4(this, _RuntimeClient_apolloClient, "f").query({
            query: gql `query runtimeFriendStatus($did: String!) {
                runtimeFriendStatus(did: $did) { ${PERSPECTIVE_EXPRESSION_FIELDS} }
            }`,
            variables: { did }
        }));
        return runtimeFriendStatus;
    }
    async friendSendMessage(did, message) {
        const { runtimeFriendSendMessage } = unwrapApolloResult(await __classPrivateFieldGet$4(this, _RuntimeClient_apolloClient, "f").mutate({
            mutation: gql `mutation runtimeFriendSendMessage($did: String!, $message: PerspectiveInput!) {
                runtimeFriendSendMessage(did: $did, message: $message)
            }`,
            variables: { did, message }
        }));
        return runtimeFriendSendMessage;
    }
    async messageInbox(filter) {
        const { runtimeMessageInbox } = unwrapApolloResult(await __classPrivateFieldGet$4(this, _RuntimeClient_apolloClient, "f").query({
            query: gql `query runtimeMessageInbox($filter: String) {
                runtimeMessageInbox(filter: $filter) { ${PERSPECTIVE_EXPRESSION_FIELDS} }
            }`,
            variables: { filter }
        }));
        return runtimeMessageInbox;
    }
    async messageOutbox(filter) {
        const { runtimeMessageOutbox } = unwrapApolloResult(await __classPrivateFieldGet$4(this, _RuntimeClient_apolloClient, "f").query({
            query: gql `query runtimeMessageOutbox($filter: String) {
                runtimeMessageOutbox(filter: $filter) { 
                    recipient,
                    message {
                        ${PERSPECTIVE_EXPRESSION_FIELDS} 
                    }
                }
            }`,
            variables: { filter }
        }));
        return runtimeMessageOutbox;
    }
    addMessageCallback(cb) {
        __classPrivateFieldGet$4(this, _RuntimeClient_messageReceivedCallbacks, "f").push(cb);
    }
    subscribeMessageReceived() {
        __classPrivateFieldGet$4(this, _RuntimeClient_apolloClient, "f").subscribe({
            query: gql ` subscription {
                runtimeMessageReceived { ${PERSPECTIVE_EXPRESSION_FIELDS} }
            }   
        `
        }).subscribe({
            next: result => {
                __classPrivateFieldGet$4(this, _RuntimeClient_messageReceivedCallbacks, "f").forEach(cb => {
                    cb(result.data.runtimeMessageReceived);
                });
            },
            error: (e) => console.error(e)
        });
    }
    addExceptionCallback(cb) {
        __classPrivateFieldGet$4(this, _RuntimeClient_exceptionOccurredCallbacks, "f").push(cb);
    }
    subscribeExceptionOccurred() {
        __classPrivateFieldGet$4(this, _RuntimeClient_apolloClient, "f").subscribe({
            query: gql ` subscription {
                exceptionOccurred {
                    title
                    message
                    type
                    addon
                }
            }`
        }).subscribe({
            next: result => {
                __classPrivateFieldGet$4(this, _RuntimeClient_exceptionOccurredCallbacks, "f").forEach(cb => {
                    cb(result.data.exceptionOccurred);
                });
            },
            error: (e) => console.error(e)
        });
    }
}
_RuntimeClient_apolloClient = new WeakMap(), _RuntimeClient_messageReceivedCallbacks = new WeakMap(), _RuntimeClient_exceptionOccurredCallbacks = new WeakMap();

var __classPrivateFieldSet$3 = function (receiver, state, value, kind, f) {
    if (kind === "m") throw new TypeError("Private method is not writable");
    if (kind === "a" && !f) throw new TypeError("Private accessor was defined without a setter");
    if (typeof state === "function" ? receiver !== state || !f : !state.has(receiver)) throw new TypeError("Cannot write private member to an object whose class did not declare it");
    return (kind === "a" ? f.call(receiver, value) : f ? f.value = value : state.set(receiver, value)), value;
};
var __classPrivateFieldGet$3 = function (receiver, state, kind, f) {
    if (kind === "a" && !f) throw new TypeError("Private accessor was defined without a getter");
    if (typeof state === "function" ? receiver !== state || !f : !state.has(receiver)) throw new TypeError("Cannot read private member from an object whose class did not declare it");
    return kind === "m" ? f : kind === "a" ? f.call(receiver) : f ? f.value : state.get(receiver);
};
var _ExpressionClient_apolloClient;
class ExpressionClient {
    constructor(client) {
        _ExpressionClient_apolloClient.set(this, void 0);
        __classPrivateFieldSet$3(this, _ExpressionClient_apolloClient, client, "f");
    }
    async get(url) {
        const { expression } = unwrapApolloResult(await __classPrivateFieldGet$3(this, _ExpressionClient_apolloClient, "f").query({
            query: gql `query expression($url: String!) {
                expression(url: $url) {
                    author
                    timestamp
                    data
                    language {
                        address
                    }
                    proof {
                        valid
                        invalid
                    }
                }
            }`,
            variables: { url }
        }));
        return expression;
    }
    async getMany(urls) {
        const { expressionMany } = unwrapApolloResult(await __classPrivateFieldGet$3(this, _ExpressionClient_apolloClient, "f").query({
            query: gql `query expressionMany($urls: [String!]!) {
                expressionMany(urls: $urls) {
                    author
                    timestamp
                    data
                    language {
                        address
                    }
                    proof {
                        valid
                        invalid
                    }
                }
            }`,
            variables: { urls }
        }));
        return expressionMany;
    }
    async getRaw(url) {
        const { expressionRaw } = unwrapApolloResult(await __classPrivateFieldGet$3(this, _ExpressionClient_apolloClient, "f").query({
            query: gql `query expressionRaw($url: String!) {
                expressionRaw(url: $url)
            }`,
            variables: { url }
        }));
        return expressionRaw;
    }
    async create(content, languageAddress) {
        content = JSON.stringify(content);
        const { expressionCreate } = unwrapApolloResult(await __classPrivateFieldGet$3(this, _ExpressionClient_apolloClient, "f").mutate({
            mutation: gql `mutation expressionCreate($content: String!, $languageAddress: String!){
                expressionCreate(content: $content, languageAddress: $languageAddress)
            }`,
            variables: { content, languageAddress }
        }));
        return expressionCreate;
    }
    async interactions(url) {
        const { expressionInteractions } = unwrapApolloResult(await __classPrivateFieldGet$3(this, _ExpressionClient_apolloClient, "f").query({
            query: gql `query expressionInteractions($url: String!) {
                expressionInteractions(url: $url) { 
                    label
                    name
                    parameters { name, type }
                }
            }`,
            variables: { url }
        }));
        return expressionInteractions;
    }
    async interact(url, interactionCall) {
        const { expressionInteract } = unwrapApolloResult(await __classPrivateFieldGet$3(this, _ExpressionClient_apolloClient, "f").mutate({
            mutation: gql `mutation expressionInteract($url: String!, $interactionCall: InteractionCall!){
                expressionInteract(url: $url, interactionCall: $interactionCall)
            }`,
            variables: { url, interactionCall }
        }));
        return expressionInteract;
    }
}
_ExpressionClient_apolloClient = new WeakMap();

var __classPrivateFieldSet$2 = function (receiver, state, value, kind, f) {
    if (kind === "m") throw new TypeError("Private method is not writable");
    if (kind === "a" && !f) throw new TypeError("Private accessor was defined without a setter");
    if (typeof state === "function" ? receiver !== state || !f : !state.has(receiver)) throw new TypeError("Cannot write private member to an object whose class did not declare it");
    return (kind === "a" ? f.call(receiver, value) : f ? f.value = value : state.set(receiver, value)), value;
};
var __classPrivateFieldGet$2 = function (receiver, state, kind, f) {
    if (kind === "a" && !f) throw new TypeError("Private accessor was defined without a getter");
    if (typeof state === "function" ? receiver !== state || !f : !state.has(receiver)) throw new TypeError("Cannot read private member from an object whose class did not declare it");
    return kind === "m" ? f : kind === "a" ? f.call(receiver) : f ? f.value : state.get(receiver);
};
var _Ad4mClient_apolloClient, _Ad4mClient_agentClient, _Ad4mClient_expressionClient, _Ad4mClient_languageClient, _Ad4mClient_neighbourhoodClient, _Ad4mClient_perspectiveClient, _Ad4mClient_runtimeClient;
/**
 * Client for the Ad4m interface wrapping GraphQL queryies
 * for convenient use in user facing code.
 *
 * Aggregates the six sub-clients:
 * AgentClient, ExpressionClient, LanguageClient,
 * NeighbourhoodClient, PerspectiveClient and RuntimeClient
 * for the respective functionality.
 */
class Ad4mClient {
    constructor(client, subscribe = true) {
        _Ad4mClient_apolloClient.set(this, void 0);
        _Ad4mClient_agentClient.set(this, void 0);
        _Ad4mClient_expressionClient.set(this, void 0);
        _Ad4mClient_languageClient.set(this, void 0);
        _Ad4mClient_neighbourhoodClient.set(this, void 0);
        _Ad4mClient_perspectiveClient.set(this, void 0);
        _Ad4mClient_runtimeClient.set(this, void 0);
        __classPrivateFieldSet$2(this, _Ad4mClient_apolloClient, client, "f");
        __classPrivateFieldSet$2(this, _Ad4mClient_agentClient, new AgentClient(__classPrivateFieldGet$2(this, _Ad4mClient_apolloClient, "f"), subscribe), "f");
        __classPrivateFieldSet$2(this, _Ad4mClient_expressionClient, new ExpressionClient(__classPrivateFieldGet$2(this, _Ad4mClient_apolloClient, "f")), "f");
        __classPrivateFieldSet$2(this, _Ad4mClient_languageClient, new LanguageClient(__classPrivateFieldGet$2(this, _Ad4mClient_apolloClient, "f")), "f");
        __classPrivateFieldSet$2(this, _Ad4mClient_neighbourhoodClient, new NeighbourhoodClient(__classPrivateFieldGet$2(this, _Ad4mClient_apolloClient, "f")), "f");
        __classPrivateFieldSet$2(this, _Ad4mClient_perspectiveClient, new PerspectiveClient(__classPrivateFieldGet$2(this, _Ad4mClient_apolloClient, "f"), subscribe), "f");
        __classPrivateFieldGet$2(this, _Ad4mClient_perspectiveClient, "f").setExpressionClient(__classPrivateFieldGet$2(this, _Ad4mClient_expressionClient, "f"));
        __classPrivateFieldGet$2(this, _Ad4mClient_perspectiveClient, "f").setNeighbourhoodClient(__classPrivateFieldGet$2(this, _Ad4mClient_neighbourhoodClient, "f"));
        __classPrivateFieldSet$2(this, _Ad4mClient_runtimeClient, new RuntimeClient(__classPrivateFieldGet$2(this, _Ad4mClient_apolloClient, "f"), subscribe), "f");
    }
    get agent() {
        return __classPrivateFieldGet$2(this, _Ad4mClient_agentClient, "f");
    }
    get expression() {
        return __classPrivateFieldGet$2(this, _Ad4mClient_expressionClient, "f");
    }
    get languages() {
        return __classPrivateFieldGet$2(this, _Ad4mClient_languageClient, "f");
    }
    get neighbourhood() {
        return __classPrivateFieldGet$2(this, _Ad4mClient_neighbourhoodClient, "f");
    }
    get perspective() {
        return __classPrivateFieldGet$2(this, _Ad4mClient_perspectiveClient, "f");
    }
    get runtime() {
        return __classPrivateFieldGet$2(this, _Ad4mClient_runtimeClient, "f");
    }
}
_Ad4mClient_apolloClient = new WeakMap(), _Ad4mClient_agentClient = new WeakMap(), _Ad4mClient_expressionClient = new WeakMap(), _Ad4mClient_languageClient = new WeakMap(), _Ad4mClient_neighbourhoodClient = new WeakMap(), _Ad4mClient_perspectiveClient = new WeakMap(), _Ad4mClient_runtimeClient = new WeakMap();

var ExceptionType;
(function (ExceptionType) {
    ExceptionType[ExceptionType["LanguageIsNotLoaded"] = 0] = "LanguageIsNotLoaded";
    ExceptionType[ExceptionType["ExpressionIsNotVerified"] = 1] = "ExpressionIsNotVerified";
    ExceptionType[ExceptionType["AgentIsUntrusted"] = 2] = "AgentIsUntrusted";
    ExceptionType[ExceptionType["CapabilityRequested"] = 3] = "CapabilityRequested";
})(ExceptionType || (ExceptionType = {}));

var __decorate$6 = function (decorators, target, key, desc) {
    var c = arguments.length, r = c < 3 ? target : desc === null ? desc = Object.getOwnPropertyDescriptor(target, key) : desc, d;
    if (typeof Reflect === "object" && typeof Reflect.decorate === "function") r = Reflect.decorate(decorators, target, key, desc);
    else for (var i = decorators.length - 1; i >= 0; i--) if (d = decorators[i]) r = (c < 3 ? d(r) : c > 3 ? d(target, key, r) : d(target, key)) || r;
    return c > 3 && r && Object.defineProperty(target, key, r), r;
};
var __metadata$6 = function (k, v) {
    if (typeof Reflect === "object" && typeof Reflect.metadata === "function") return Reflect.metadata(k, v);
};
// Expression address + unique Language ID = global expression URL
let ExpressionRef = class ExpressionRef {
    constructor(lang, expr) {
        this.language = lang;
        this.expression = expr;
    }
};
__decorate$6([
    Field(),
    __metadata$6("design:type", LanguageRef)
], ExpressionRef.prototype, "language", void 0);
__decorate$6([
    Field(),
    __metadata$6("design:type", String)
], ExpressionRef.prototype, "expression", void 0);
ExpressionRef = __decorate$6([
    ObjectType(),
    __metadata$6("design:paramtypes", [LanguageRef, String])
], ExpressionRef);

var __decorate$5 = function (decorators, target, key, desc) {
    var c = arguments.length, r = c < 3 ? target : desc === null ? desc = Object.getOwnPropertyDescriptor(target, key) : desc, d;
    if (typeof Reflect === "object" && typeof Reflect.decorate === "function") r = Reflect.decorate(decorators, target, key, desc);
    else for (var i = decorators.length - 1; i >= 0; i--) if (d = decorators[i]) r = (c < 3 ? d(r) : c > 3 ? d(target, key, r) : d(target, key)) || r;
    return c > 3 && r && Object.defineProperty(target, key, r), r;
};
var __metadata$5 = function (k, v) {
    if (typeof Reflect === "object" && typeof Reflect.metadata === "function") return Reflect.metadata(k, v);
};
let InteractionParameter = class InteractionParameter {
};
__decorate$5([
    Field(),
    __metadata$5("design:type", String)
], InteractionParameter.prototype, "name", void 0);
__decorate$5([
    Field(),
    __metadata$5("design:type", String)
], InteractionParameter.prototype, "type", void 0);
InteractionParameter = __decorate$5([
    ObjectType()
], InteractionParameter);
let InteractionMeta = class InteractionMeta {
};
__decorate$5([
    Field(),
    __metadata$5("design:type", String)
], InteractionMeta.prototype, "label", void 0);
__decorate$5([
    Field(),
    __metadata$5("design:type", String)
], InteractionMeta.prototype, "name", void 0);
__decorate$5([
    Field(),
    __metadata$5("design:type", Array)
], InteractionMeta.prototype, "parameters", void 0);
InteractionMeta = __decorate$5([
    ObjectType()
], InteractionMeta);
let InteractionCall = class InteractionCall {
    get parameters() {
        return JSON.parse(this.parametersStringified);
    }
    constructor(name, parameters) {
        this.name = name;
        this.parametersStringified = JSON.stringify(parameters);
    }
};
__decorate$5([
    Field(),
    __metadata$5("design:type", String)
], InteractionCall.prototype, "name", void 0);
__decorate$5([
    Field(),
    __metadata$5("design:type", String)
], InteractionCall.prototype, "parametersStringified", void 0);
InteractionCall = __decorate$5([
    InputType(),
    __metadata$5("design:paramtypes", [String, Object])
], InteractionCall);
let OnlineAgent = class OnlineAgent {
};
__decorate$5([
    Field(),
    __metadata$5("design:type", String)
], OnlineAgent.prototype, "did", void 0);
__decorate$5([
    Field(),
    __metadata$5("design:type", PerspectiveExpression)
], OnlineAgent.prototype, "status", void 0);
OnlineAgent = __decorate$5([
    ObjectType()
], OnlineAgent);

var __decorate$4 = function (decorators, target, key, desc) {
    var c = arguments.length, r = c < 3 ? target : desc === null ? desc = Object.getOwnPropertyDescriptor(target, key) : desc, d;
    if (typeof Reflect === "object" && typeof Reflect.decorate === "function") r = Reflect.decorate(decorators, target, key, desc);
    else for (var i = decorators.length - 1; i >= 0; i--) if (d = decorators[i]) r = (c < 3 ? d(r) : c > 3 ? d(target, key, r) : d(target, key)) || r;
    return c > 3 && r && Object.defineProperty(target, key, r), r;
};
var __metadata$4 = function (k, v) {
    if (typeof Reflect === "object" && typeof Reflect.metadata === "function") return Reflect.metadata(k, v);
};
let LanguageHandle = class LanguageHandle {
};
__decorate$4([
    Field(),
    __metadata$4("design:type", String)
], LanguageHandle.prototype, "name", void 0);
__decorate$4([
    Field(),
    __metadata$4("design:type", String)
], LanguageHandle.prototype, "address", void 0);
__decorate$4([
    Field(),
    __metadata$4("design:type", String)
], LanguageHandle.prototype, "settings", void 0);
__decorate$4([
    Field(),
    __metadata$4("design:type", Icon)
], LanguageHandle.prototype, "icon", void 0);
__decorate$4([
    Field(),
    __metadata$4("design:type", Icon)
], LanguageHandle.prototype, "constructorIcon", void 0);
__decorate$4([
    Field(),
    __metadata$4("design:type", Icon)
], LanguageHandle.prototype, "settingsIcon", void 0);
LanguageHandle = __decorate$4([
    ObjectType()
], LanguageHandle);

var __decorate$3 = function (decorators, target, key, desc) {
    var c = arguments.length, r = c < 3 ? target : desc === null ? desc = Object.getOwnPropertyDescriptor(target, key) : desc, d;
    if (typeof Reflect === "object" && typeof Reflect.decorate === "function") r = Reflect.decorate(decorators, target, key, desc);
    else for (var i = decorators.length - 1; i >= 0; i--) if (d = decorators[i]) r = (c < 3 ? d(r) : c > 3 ? d(target, key, r) : d(target, key)) || r;
    return c > 3 && r && Object.defineProperty(target, key, r), r;
};
var __metadata$3 = function (k, v) {
    if (typeof Reflect === "object" && typeof Reflect.metadata === "function") return Reflect.metadata(k, v);
};
let LanguageMeta = class LanguageMeta {
};
__decorate$3([
    Field(),
    __metadata$3("design:type", String)
], LanguageMeta.prototype, "name", void 0);
__decorate$3([
    Field(),
    __metadata$3("design:type", String)
], LanguageMeta.prototype, "address", void 0);
__decorate$3([
    Field(),
    __metadata$3("design:type", String)
], LanguageMeta.prototype, "description", void 0);
__decorate$3([
    Field(),
    __metadata$3("design:type", String)
], LanguageMeta.prototype, "author", void 0);
__decorate$3([
    Field(),
    __metadata$3("design:type", Boolean)
], LanguageMeta.prototype, "templated", void 0);
__decorate$3([
    Field(),
    __metadata$3("design:type", String)
], LanguageMeta.prototype, "templateSourceLanguageAddress", void 0);
__decorate$3([
    Field(),
    __metadata$3("design:type", String)
], LanguageMeta.prototype, "templateAppliedParams", void 0);
__decorate$3([
    Field(),
    __metadata$3("design:type", Array)
], LanguageMeta.prototype, "possibleTemplateParams", void 0);
__decorate$3([
    Field(),
    __metadata$3("design:type", String)
], LanguageMeta.prototype, "sourceCodeLink", void 0);
LanguageMeta = __decorate$3([
    ObjectType()
], LanguageMeta);
let LanguageMetaInput = class LanguageMetaInput {
    constructor(name, description) {
        this.name = name;
        this.description = description;
        if (!this.description)
            this.description = "";
    }
};
__decorate$3([
    Field(),
    __metadata$3("design:type", String)
], LanguageMetaInput.prototype, "name", void 0);
__decorate$3([
    Field(),
    __metadata$3("design:type", String)
], LanguageMetaInput.prototype, "description", void 0);
__decorate$3([
    Field(),
    __metadata$3("design:type", Array)
], LanguageMetaInput.prototype, "possibleTemplateParams", void 0);
__decorate$3([
    Field(),
    __metadata$3("design:type", String)
], LanguageMetaInput.prototype, "sourceCodeLink", void 0);
LanguageMetaInput = __decorate$3([
    InputType(),
    __metadata$3("design:paramtypes", [String, String])
], LanguageMetaInput);
class LanguageExpression extends ExpressionGeneric() {
}

var __decorate$2 = function (decorators, target, key, desc) {
    var c = arguments.length, r = c < 3 ? target : desc === null ? desc = Object.getOwnPropertyDescriptor(target, key) : desc, d;
    if (typeof Reflect === "object" && typeof Reflect.decorate === "function") r = Reflect.decorate(decorators, target, key, desc);
    else for (var i = decorators.length - 1; i >= 0; i--) if (d = decorators[i]) r = (c < 3 ? d(r) : c > 3 ? d(target, key, r) : d(target, key)) || r;
    return c > 3 && r && Object.defineProperty(target, key, r), r;
};
var __metadata$2 = function (k, v) {
    if (typeof Reflect === "object" && typeof Reflect.metadata === "function") return Reflect.metadata(k, v);
};
let Neighbourhood = class Neighbourhood {
    constructor(linkLanguage, meta) {
        this.linkLanguage = linkLanguage;
        this.meta = meta;
    }
};
__decorate$2([
    Field(),
    __metadata$2("design:type", String)
], Neighbourhood.prototype, "linkLanguage", void 0);
__decorate$2([
    Field(),
    __metadata$2("design:type", Perspective)
], Neighbourhood.prototype, "meta", void 0);
Neighbourhood = __decorate$2([
    ObjectType(),
    __metadata$2("design:paramtypes", [String, Perspective])
], Neighbourhood);
let NeighbourhoodExpression = class NeighbourhoodExpression extends ExpressionGeneric() {
};
NeighbourhoodExpression = __decorate$2([
    ObjectType()
], NeighbourhoodExpression);

var __decorate$1 = function (decorators, target, key, desc) {
    var c = arguments.length, r = c < 3 ? target : desc === null ? desc = Object.getOwnPropertyDescriptor(target, key) : desc, d;
    if (typeof Reflect === "object" && typeof Reflect.decorate === "function") r = Reflect.decorate(decorators, target, key, desc);
    else for (var i = decorators.length - 1; i >= 0; i--) if (d = decorators[i]) r = (c < 3 ? d(r) : c > 3 ? d(target, key, r) : d(target, key)) || r;
    return c > 3 && r && Object.defineProperty(target, key, r), r;
};
var __metadata$1 = function (k, v) {
    if (typeof Reflect === "object" && typeof Reflect.metadata === "function") return Reflect.metadata(k, v);
};
var PerspectiveState;
(function (PerspectiveState) {
    PerspectiveState["Private"] = "Private";
    PerspectiveState["NeighbourhoodJoinInitiated"] = "NeighbourhoodJoinInitiated";
    PerspectiveState["LinkLanguageFailedToInstall"] = "LinkLanguageFailedToInstall";
    PerspectiveState["LinkLanguageInstalledButNotSynced"] = "LinkLanguageInstalledButNotSynced";
    PerspectiveState["Synced"] = "Synced";
})(PerspectiveState || (PerspectiveState = {}));
// This type is used in the GraphQL interface to reference a mutable
// prespective that is implemented locally by the Ad4m runtime.
// The UUID is used in mutations to identify the perspective that gets mutated.
let PerspectiveHandle = class PerspectiveHandle {
    constructor(uuid, name, state) {
        this.uuid = uuid;
        this.name = name;
        if (state) {
            this.state = state;
        }
        else {
            this.state = PerspectiveState.Private;
        }
    }
};
__decorate$1([
    Field(),
    __metadata$1("design:type", String)
], PerspectiveHandle.prototype, "uuid", void 0);
__decorate$1([
    Field(),
    __metadata$1("design:type", String)
], PerspectiveHandle.prototype, "name", void 0);
__decorate$1([
    Field(),
    __metadata$1("design:type", String)
], PerspectiveHandle.prototype, "state", void 0);
__decorate$1([
    Field(),
    __metadata$1("design:type", String)
], PerspectiveHandle.prototype, "sharedUrl", void 0);
__decorate$1([
    Field(),
    __metadata$1("design:type", NeighbourhoodExpression)
], PerspectiveHandle.prototype, "neighbourhood", void 0);
PerspectiveHandle = __decorate$1([
    ObjectType(),
    __metadata$1("design:paramtypes", [String, String, String])
], PerspectiveHandle);

var __decorate = function (decorators, target, key, desc) {
    var c = arguments.length, r = c < 3 ? target : desc === null ? desc = Object.getOwnPropertyDescriptor(target, key) : desc, d;
    if (typeof Reflect === "object" && typeof Reflect.decorate === "function") r = Reflect.decorate(decorators, target, key, desc);
    else for (var i = decorators.length - 1; i >= 0; i--) if (d = decorators[i]) r = (c < 3 ? d(r) : c > 3 ? d(target, key, r) : d(target, key)) || r;
    return c > 3 && r && Object.defineProperty(target, key, r), r;
};
var __metadata = function (k, v) {
    if (typeof Reflect === "object" && typeof Reflect.metadata === "function") return Reflect.metadata(k, v);
};
let PerspectiveDiff = class PerspectiveDiff {
};
__decorate([
    Field(),
    __metadata("design:type", Array)
], PerspectiveDiff.prototype, "additions", void 0);
__decorate([
    Field(),
    __metadata("design:type", Array)
], PerspectiveDiff.prototype, "removals", void 0);
PerspectiveDiff = __decorate([
    ObjectType()
], PerspectiveDiff);
let PerspectiveDiffExpression = class PerspectiveDiffExpression extends ExpressionGeneric() {
};
PerspectiveDiffExpression = __decorate([
    ObjectType()
], PerspectiveDiffExpression);

/* App.svelte generated by Svelte v4.2.1 */

function add_css(target) {
	append_styles(target, "svelte-232lbh", "main.svelte-232lbh{padding:1em;max-width:240px;margin:0 auto;max-width:none}@media(min-width: 640px){main.svelte-232lbh{max-width:none}}@media only screen and (max-width: 600px){main.svelte-232lbh{text-align:center}}");
}

// (116:4) {:else}
function create_else_block_1(ctx) {
	let h2;
	let t1;
	let br;
	let t2;
	let label;
	let t3;
	let input;
	let t4;
	let button;
	let current;
	let mounted;
	let dispose;

	label = new Button.Label({
			props: {
				$$slots: { default: [create_default_slot_8] },
				$$scope: { ctx }
			}
		});

	button = new Button({
			props: {
				variant: "raised",
				color: "primary",
				$$slots: { default: [create_default_slot_6] },
				$$scope: { ctx }
			}
		});

	button.$on("click", /*unlock*/ ctx[5]);

	return {
		c() {
			h2 = element("h2");
			h2.textContent = "Please unlock your ad4m agent...";
			t1 = space();
			br = element("br");
			t2 = space();
			create_component(label.$$.fragment);
			t3 = space();
			input = element("input");
			t4 = space();
			create_component(button.$$.fragment);
			attr(input, "variant", "raised");
			attr(input, "color", "primary");
			attr(input, "type", "password");
		},
		m(target, anchor) {
			insert(target, h2, anchor);
			insert(target, t1, anchor);
			insert(target, br, anchor);
			insert(target, t2, anchor);
			mount_component(label, target, anchor);
			insert(target, t3, anchor);
			insert(target, input, anchor);
			set_input_value(input, /*password*/ ctx[1]);
			insert(target, t4, anchor);
			mount_component(button, target, anchor);
			current = true;

			if (!mounted) {
				dispose = listen(input, "input", /*input_input_handler*/ ctx[6]);
				mounted = true;
			}
		},
		p(ctx, dirty) {
			const label_changes = {};

			if (dirty & /*$$scope*/ 4096) {
				label_changes.$$scope = { dirty, ctx };
			}

			label.$set(label_changes);

			if (dirty & /*password*/ 2 && input.value !== /*password*/ ctx[1]) {
				set_input_value(input, /*password*/ ctx[1]);
			}

			const button_changes = {};

			if (dirty & /*$$scope*/ 4096) {
				button_changes.$$scope = { dirty, ctx };
			}

			button.$set(button_changes);
		},
		i(local) {
			if (current) return;
			transition_in(label.$$.fragment, local);
			transition_in(button.$$.fragment, local);
			current = true;
		},
		o(local) {
			transition_out(label.$$.fragment, local);
			transition_out(button.$$.fragment, local);
			current = false;
		},
		d(detaching) {
			if (detaching) {
				detach(h2);
				detach(t1);
				detach(br);
				detach(t2);
				detach(t3);
				detach(input);
				detach(t4);
			}

			destroy_component(label, detaching);
			destroy_component(button, detaching);
			mounted = false;
			dispose();
		}
	};
}

// (96:4) {#if signedIn==true}
function create_if_block(ctx) {
	let current_block_type_index;
	let if_block;
	let if_block_anchor;
	let current;
	const if_block_creators = [create_if_block_1, create_else_block];
	const if_blocks = [];

	function select_block_type_1(ctx, dirty) {
		if (/*currentAccount*/ ctx[0] == undefined) return 0;
		return 1;
	}

	current_block_type_index = select_block_type_1(ctx);
	if_block = if_blocks[current_block_type_index] = if_block_creators[current_block_type_index](ctx);

	return {
		c() {
			if_block.c();
			if_block_anchor = empty();
		},
		m(target, anchor) {
			if_blocks[current_block_type_index].m(target, anchor);
			insert(target, if_block_anchor, anchor);
			current = true;
		},
		p(ctx, dirty) {
			let previous_block_index = current_block_type_index;
			current_block_type_index = select_block_type_1(ctx);

			if (current_block_type_index === previous_block_index) {
				if_blocks[current_block_type_index].p(ctx, dirty);
			} else {
				group_outros();

				transition_out(if_blocks[previous_block_index], 1, 1, () => {
					if_blocks[previous_block_index] = null;
				});

				check_outros();
				if_block = if_blocks[current_block_type_index];

				if (!if_block) {
					if_block = if_blocks[current_block_type_index] = if_block_creators[current_block_type_index](ctx);
					if_block.c();
				} else {
					if_block.p(ctx, dirty);
				}

				transition_in(if_block, 1);
				if_block.m(if_block_anchor.parentNode, if_block_anchor);
			}
		},
		i(local) {
			if (current) return;
			transition_in(if_block);
			current = true;
		},
		o(local) {
			transition_out(if_block);
			current = false;
		},
		d(detaching) {
			if (detaching) {
				detach(if_block_anchor);
			}

			if_blocks[current_block_type_index].d(detaching);
		}
	};
}

// (119:8) <Label>
function create_default_slot_8(ctx) {
	let t;

	return {
		c() {
			t = text("Password:");
		},
		m(target, anchor) {
			insert(target, t, anchor);
		},
		d(detaching) {
			if (detaching) {
				detach(t);
			}
		}
	};
}

// (122:12) <Label>
function create_default_slot_7(ctx) {
	let t;

	return {
		c() {
			t = text("Unlock");
		},
		m(target, anchor) {
			insert(target, t, anchor);
		},
		d(detaching) {
			if (detaching) {
				detach(t);
			}
		}
	};
}

// (121:8) <Button on:click={unlock} variant="raised" color="primary">
function create_default_slot_6(ctx) {
	let label;
	let current;

	label = new Button.Label({
			props: {
				$$slots: { default: [create_default_slot_7] },
				$$scope: { ctx }
			}
		});

	return {
		c() {
			create_component(label.$$.fragment);
		},
		m(target, anchor) {
			mount_component(label, target, anchor);
			current = true;
		},
		p(ctx, dirty) {
			const label_changes = {};

			if (dirty & /*$$scope*/ 4096) {
				label_changes.$$scope = { dirty, ctx };
			}

			label.$set(label_changes);
		},
		i(local) {
			if (current) return;
			transition_in(label.$$.fragment, local);
			current = true;
		},
		o(local) {
			transition_out(label.$$.fragment, local);
			current = false;
		},
		d(detaching) {
			destroy_component(label, detaching);
		}
	};
}

// (110:8) {:else}
function create_else_block(ctx) {
	let h2;
	let t1;
	let button;
	let current;

	button = new Button({
			props: {
				variant: "raised",
				color: "primary",
				$$slots: { default: [create_default_slot_4] },
				$$scope: { ctx }
			}
		});

	button.$on("click", /*signDid*/ ctx[4]);

	return {
		c() {
			h2 = element("h2");
			h2.textContent = "Create link";
			t1 = space();
			create_component(button.$$.fragment);
		},
		m(target, anchor) {
			insert(target, h2, anchor);
			insert(target, t1, anchor);
			mount_component(button, target, anchor);
			current = true;
		},
		p(ctx, dirty) {
			const button_changes = {};

			if (dirty & /*$$scope*/ 4096) {
				button_changes.$$scope = { dirty, ctx };
			}

			button.$set(button_changes);
		},
		i(local) {
			if (current) return;
			transition_in(button.$$.fragment, local);
			current = true;
		},
		o(local) {
			transition_out(button.$$.fragment, local);
			current = false;
		},
		d(detaching) {
			if (detaching) {
				detach(h2);
				detach(t1);
			}

			destroy_component(button, detaching);
		}
	};
}

// (97:8) {#if currentAccount==undefined}
function create_if_block_1(ctx) {
	let h2;
	let t1;
	let p;
	let h30;
	let t4;
	let button0;
	let t5;
	let br;
	let t6;
	let h31;
	let t8;
	let button1;
	let current;

	button0 = new Button({
			props: {
				variant: "raised",
				color: "primary",
				$$slots: { default: [create_default_slot_2] },
				$$scope: { ctx }
			}
		});

	button0.$on("click", /*connect*/ ctx[3]);

	button1 = new Button({
			props: {
				variant: "raised",
				color: "primary",
				$$slots: { default: [create_default_slot] },
				$$scope: { ctx }
			}
		});

	button1.$on("click", walletConnectConnect);

	return {
		c() {
			h2 = element("h2");
			h2.textContent = "Get Started";
			t1 = space();
			p = element("p");
			p.textContent = "Welcome to AD4M, thanks for being here. Lets connect all the dapps :)\n            ";
			h30 = element("h3");
			h30.textContent = "Sign into MetaMask ";
			t4 = space();
			create_component(button0.$$.fragment);
			t5 = space();
			br = element("br");
			t6 = space();
			h31 = element("h3");
			h31.textContent = "Sign into WalletConnect";
			t8 = space();
			create_component(button1.$$.fragment);
		},
		m(target, anchor) {
			insert(target, h2, anchor);
			insert(target, t1, anchor);
			insert(target, p, anchor);
			insert(target, h30, anchor);
			insert(target, t4, anchor);
			mount_component(button0, target, anchor);
			insert(target, t5, anchor);
			insert(target, br, anchor);
			insert(target, t6, anchor);
			insert(target, h31, anchor);
			insert(target, t8, anchor);
			mount_component(button1, target, anchor);
			current = true;
		},
		p(ctx, dirty) {
			const button0_changes = {};

			if (dirty & /*$$scope*/ 4096) {
				button0_changes.$$scope = { dirty, ctx };
			}

			button0.$set(button0_changes);
			const button1_changes = {};

			if (dirty & /*$$scope*/ 4096) {
				button1_changes.$$scope = { dirty, ctx };
			}

			button1.$set(button1_changes);
		},
		i(local) {
			if (current) return;
			transition_in(button0.$$.fragment, local);
			transition_in(button1.$$.fragment, local);
			current = true;
		},
		o(local) {
			transition_out(button0.$$.fragment, local);
			transition_out(button1.$$.fragment, local);
			current = false;
		},
		d(detaching) {
			if (detaching) {
				detach(h2);
				detach(t1);
				detach(p);
				detach(h30);
				detach(t4);
				detach(t5);
				detach(br);
				detach(t6);
				detach(h31);
				detach(t8);
			}

			destroy_component(button0, detaching);
			destroy_component(button1, detaching);
		}
	};
}

// (113:16) <Label>
function create_default_slot_5(ctx) {
	let t;

	return {
		c() {
			t = text("Sign");
		},
		m(target, anchor) {
			insert(target, t, anchor);
		},
		d(detaching) {
			if (detaching) {
				detach(t);
			}
		}
	};
}

// (112:12) <Button on:click={signDid} variant="raised" color="primary">
function create_default_slot_4(ctx) {
	let label;
	let current;

	label = new Button.Label({
			props: {
				$$slots: { default: [create_default_slot_5] },
				$$scope: { ctx }
			}
		});

	return {
		c() {
			create_component(label.$$.fragment);
		},
		m(target, anchor) {
			mount_component(label, target, anchor);
			current = true;
		},
		p(ctx, dirty) {
			const label_changes = {};

			if (dirty & /*$$scope*/ 4096) {
				label_changes.$$scope = { dirty, ctx };
			}

			label.$set(label_changes);
		},
		i(local) {
			if (current) return;
			transition_in(label.$$.fragment, local);
			current = true;
		},
		o(local) {
			transition_out(label.$$.fragment, local);
			current = false;
		},
		d(detaching) {
			destroy_component(label, detaching);
		}
	};
}

// (103:16) <Label>
function create_default_slot_3(ctx) {
	let t;

	return {
		c() {
			t = text("Connect MetaMask");
		},
		m(target, anchor) {
			insert(target, t, anchor);
		},
		d(detaching) {
			if (detaching) {
				detach(t);
			}
		}
	};
}

// (102:12) <Button on:click={connect} variant="raised" color="primary">
function create_default_slot_2(ctx) {
	let label;
	let current;

	label = new Button.Label({
			props: {
				$$slots: { default: [create_default_slot_3] },
				$$scope: { ctx }
			}
		});

	return {
		c() {
			create_component(label.$$.fragment);
		},
		m(target, anchor) {
			mount_component(label, target, anchor);
			current = true;
		},
		p(ctx, dirty) {
			const label_changes = {};

			if (dirty & /*$$scope*/ 4096) {
				label_changes.$$scope = { dirty, ctx };
			}

			label.$set(label_changes);
		},
		i(local) {
			if (current) return;
			transition_in(label.$$.fragment, local);
			current = true;
		},
		o(local) {
			transition_out(label.$$.fragment, local);
			current = false;
		},
		d(detaching) {
			destroy_component(label, detaching);
		}
	};
}

// (108:16) <Label>
function create_default_slot_1(ctx) {
	let t;

	return {
		c() {
			t = text("Connect WalletConnect");
		},
		m(target, anchor) {
			insert(target, t, anchor);
		},
		d(detaching) {
			if (detaching) {
				detach(t);
			}
		}
	};
}

// (107:12) <Button on:click={walletConnectConnect} variant="raised" color="primary">
function create_default_slot(ctx) {
	let label;
	let current;

	label = new Button.Label({
			props: {
				$$slots: { default: [create_default_slot_1] },
				$$scope: { ctx }
			}
		});

	return {
		c() {
			create_component(label.$$.fragment);
		},
		m(target, anchor) {
			mount_component(label, target, anchor);
			current = true;
		},
		p(ctx, dirty) {
			const label_changes = {};

			if (dirty & /*$$scope*/ 4096) {
				label_changes.$$scope = { dirty, ctx };
			}

			label.$set(label_changes);
		},
		i(local) {
			if (current) return;
			transition_in(label.$$.fragment, local);
			current = true;
		},
		o(local) {
			transition_out(label.$$.fragment, local);
			current = false;
		},
		d(detaching) {
			destroy_component(label, detaching);
		}
	};
}

function create_fragment(ctx) {
	let main;
	let notificationdisplay;
	let t;
	let current_block_type_index;
	let if_block;
	let current;
	notificationdisplay = new svelteNotifications.NotificationDisplay({});
	const if_block_creators = [create_if_block, create_else_block_1];
	const if_blocks = [];

	function select_block_type(ctx, dirty) {
		if (/*signedIn*/ ctx[2] == true) return 0;
		return 1;
	}

	current_block_type_index = select_block_type(ctx);
	if_block = if_blocks[current_block_type_index] = if_block_creators[current_block_type_index](ctx);

	return {
		c() {
			main = element("main");
			create_component(notificationdisplay.$$.fragment);
			t = space();
			if_block.c();
			attr(main, "class", "svelte-232lbh");
		},
		m(target, anchor) {
			insert(target, main, anchor);
			mount_component(notificationdisplay, main, null);
			append(main, t);
			if_blocks[current_block_type_index].m(main, null);
			current = true;
		},
		p(ctx, [dirty]) {
			let previous_block_index = current_block_type_index;
			current_block_type_index = select_block_type(ctx);

			if (current_block_type_index === previous_block_index) {
				if_blocks[current_block_type_index].p(ctx, dirty);
			} else {
				group_outros();

				transition_out(if_blocks[previous_block_index], 1, 1, () => {
					if_blocks[previous_block_index] = null;
				});

				check_outros();
				if_block = if_blocks[current_block_type_index];

				if (!if_block) {
					if_block = if_blocks[current_block_type_index] = if_block_creators[current_block_type_index](ctx);
					if_block.c();
				} else {
					if_block.p(ctx, dirty);
				}

				transition_in(if_block, 1);
				if_block.m(main, null);
			}
		},
		i(local) {
			if (current) return;
			transition_in(notificationdisplay.$$.fragment, local);
			transition_in(if_block);
			current = true;
		},
		o(local) {
			transition_out(notificationdisplay.$$.fragment, local);
			transition_out(if_block);
			current = false;
		},
		d(detaching) {
			if (detaching) {
				detach(main);
			}

			destroy_component(notificationdisplay);
			if_blocks[current_block_type_index].d();
		}
	};
}

function walletConnectConnect() {
	console.log("Wanting to connect to wallet connect");
}

function instance($$self, $$props, $$invalidate) {
	let currentAccount = undefined;
	let password = undefined;

	function constructApolloClient(port) {
		return new ApolloClient({
				link: new WebSocketLink({
						uri: `ws://localhost:${port}/graphql`,
						options: { reconnect: true }
					}),
				cache: new InMemoryCache()
			});
	}

	const apolloClient = constructApolloClient(4000);
	const ad4mClient = new Ad4mClient(apolloClient);
	let signedIn = false;
	let did = null;

	ad4mClient.agent.status().then(result => {
		$$invalidate(2, signedIn = result.isUnlocked);
		did = result.did;
	});

	function handleAccountsChanged(accounts) {
		console.log('Accounts changed: ' + accounts);

		if (accounts.length === 0) {
			// MetaMask is locked or the user has not connected any accounts
			console.log('Please connect to MetaMask.');
		} else if (accounts[0] !== currentAccount && accounts[0]) {
			$$invalidate(0, currentAccount = accounts[0]);
		}
	}

	function connect() {
		connectWallet(handleAccountsChanged);
	}

	function signDid() {
		//TODO: check that EP has not already been created with current eth address
		sign(currentAccount, did).then(result => {
			const { data, signature, r, s, v } = result;

			//Then sign eth address with did
			ad4mClient.agent.entanglementProofPreFlight(ethereum.selectedAddress, "ethereum").then(result => {
				//Add the signed did to the proof
				result.didSignedByDeviceKey = signature;

				delete result["__typename"];
				console.log("Sending", result);

				//Save proof into ad4m
				ad4mClient.agent.addEntanglementProofs([result]).then(result => {
					console.log("Added EP with result", result);
					svelteNotifications.notifier.success('Ethereum Proof Generated!');
				});
			});
		});
	}

	function unlock() {
		ad4mClient.agent.unlock(password).then(result => {
			console.log("Got unlock result", result);

			if (result.isUnlocked == true) {
				$$invalidate(2, signedIn = true);
			} else {
				$$invalidate(2, signedIn = false);
				svelteNotifications.notifier.error('Incorrect Password!');
			}
		});
	}

	startEthereum(handleAccountsChanged);

	function input_input_handler() {
		password = this.value;
		$$invalidate(1, password);
	}

	return [
		currentAccount,
		password,
		signedIn,
		connect,
		signDid,
		unlock,
		input_input_handler
	];
}

class App extends SvelteComponent {
	constructor(options) {
		super();
		init(this, options, instance, create_fragment, safe_not_equal, {}, add_css);
	}
}

const app = new App({
	target: document.body,
});

module.exports = app;
