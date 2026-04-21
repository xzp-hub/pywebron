import { Cn as filter, Dn as inherits, E as CompoundPath, En as indexOf, Et as vector_exports, Fn as isObject, Gn as mixin, Hn as map, Jn as reduce, Ln as isString, M as ZRImage, Mt as matrix_exports, O as encodeHTML, On as isArray, Sn as extend, T as truncateText, Un as merge, Y as BoundingRect, _n as curry, ar as setPlatformAPI, bn as each, d as Group, jn as isFunction, mn as clone, n as brushSingle, nn as color_exports, or as __exportAll, pn as bind, tn as env, tr as util_exports$1, u as zrender_exports, v as ZRText, vn as defaults, x as Rect } from "./graphic-Bd3Ikl7D.js";
import { $ as registerCoordinateSystem, $n as makeImage, A as getStackedDimension, An as registerLocale, B as PRIORITY, Ci as getPrecisionSafe, Cn as getTooltipMarker, Cr as Polygon, Di as nice, Dn as getTextRect, Dr as Circle, Ei as linearMap, Er as Ellipse, Et as ChartView, Fi as remRadian, G as disconnect, Gn as createIcon, H as dataTool, Ii as round, In as createTextStyle$1, J as getInstanceByDom, Jn as getShapeClass, K as dispose, Kn as extendPath, M as SeriesData, Mi as quantity, Mr as enableHoverEmphasis, Ni as quantityExponent, Nn as Model, Oi as numericToNumber, On as format, Ot as ComponentView, Pi as reformIntervals, Q as registerAction, Si as getPrecision, Sr as Polyline, Ti as isRadianAroundZero, Tn as toCamelCase, Tr as Sector, Tt as throttle, U as dependencies, Un as clipPointsByRect, V as connect, W as disConnect, Wn as clipRectByRect, Wr as getECData, X as getMap, Y as getInstanceById, Yn as getTransform, Z as init, _i as MAX_SAFE_INTEGER, _n as addCommas, _r as RadialGradient, at as registerPreprocessor, bi as getPercentWithPrecision, bn as formatTime, br as BezierCurve, c as createScaleByModel, ct as registerTransform, dt as setCanvasCreator, er as makePath, et as registerLayout, ft as version, hr as IncrementalDisplayable, it as registerPostUpdate, j as isDimensionStacked, ji as quantile, k as enableDataStack, ki as parseDate, kt as SeriesModel, lt as registerUpdateLifecycle, m as niceScaleExtent, mr as updateProps, mt as createSymbol, n as parseGeoJSON, nr as registerShape, nt as registerMap, o as use, on as ComponentModel, ot as registerProcessor, pn as getLayoutRect, q as getCoordinateSystemDimensions, qn as extendShape, rr as resizePath, rt as registerPostInit, s as AxisModelCommonMixin, st as registerTheme, t as Axis, tr as mergePath, tt as registerLoading, ur as initProps, ut as registerVisual, vi as asc, vn as capitalFirst, vr as LinearGradient, wi as isNumeric, wn as normalizeCssArray, wr as Ring, xi as getPixelPrecision, xn as formatTpl, xr as Line, yr as Arc } from "./Axis--lthWc4Q.js";
import { n as createDimensions, t as createSeriesData } from "./createSeriesData-BewE4TyZ.js";
//#region node_modules/echarts/lib/export/api/helper.js
/**
* AUTO-GENERATED FILE. DO NOT MODIFY.
*/
/**
* This module exposes helper functions for developing extensions.
*/
var helper_exports = /* @__PURE__ */ __exportAll({
	createDimensions: () => createDimensions,
	createList: () => createList,
	createScale: () => createScale,
	createSymbol: () => createSymbol,
	createTextStyle: () => createTextStyle,
	dataStack: () => dataStack,
	enableHoverEmphasis: () => enableHoverEmphasis,
	getECData: () => getECData,
	getLayoutRect: () => getLayoutRect,
	mixinAxisModelCommonMethods: () => mixinAxisModelCommonMethods
});
/**
* Create a multi dimension List structure from seriesModel.
*/
function createList(seriesModel) {
	return createSeriesData(null, seriesModel);
}
var dataStack = {
	isDimensionStacked,
	enableDataStack,
	getStackedDimension
};
/**
* Create scale
* @param {Array.<number>} dataExtent
* @param {Object|module:echarts/Model} option If `optoin.type`
*        is secified, it can only be `'value'` currently.
*/
function createScale(dataExtent, option) {
	var axisModel = option;
	if (!(option instanceof Model)) axisModel = new Model(option);
	var scale = createScaleByModel(axisModel);
	scale.setExtent(dataExtent[0], dataExtent[1]);
	niceScaleExtent(scale, axisModel);
	return scale;
}
/**
* Mixin common methods to axis model,
*
* Include methods
* `getFormattedLabels() => Array.<string>`
* `getCategories() => Array.<string>`
* `getMin(origin: boolean) => number`
* `getMax(origin: boolean) => number`
* `getNeedCrossZero() => boolean`
*/
function mixinAxisModelCommonMethods(Model) {
	mixin(Model, AxisModelCommonMixin);
}
function createTextStyle(textStyleModel, opts) {
	opts = opts || {};
	return createTextStyle$1(textStyleModel, null, null, opts.state !== "normal");
}
//#endregion
//#region node_modules/echarts/lib/export/api/number.js
var number_exports = /* @__PURE__ */ __exportAll({
	MAX_SAFE_INTEGER: () => MAX_SAFE_INTEGER,
	asc: () => asc,
	getPercentWithPrecision: () => getPercentWithPrecision,
	getPixelPrecision: () => getPixelPrecision,
	getPrecision: () => getPrecision,
	getPrecisionSafe: () => getPrecisionSafe,
	isNumeric: () => isNumeric,
	isRadianAroundZero: () => isRadianAroundZero,
	linearMap: () => linearMap,
	nice: () => nice,
	numericToNumber: () => numericToNumber,
	parseDate: () => parseDate,
	quantile: () => quantile,
	quantity: () => quantity,
	quantityExponent: () => quantityExponent,
	reformIntervals: () => reformIntervals,
	remRadian: () => remRadian,
	round: () => round
});
//#endregion
//#region node_modules/echarts/lib/export/api/time.js
var time_exports = /* @__PURE__ */ __exportAll({
	format: () => format,
	parse: () => parseDate
});
//#endregion
//#region node_modules/echarts/lib/export/api/graphic.js
var graphic_exports = /* @__PURE__ */ __exportAll({
	Arc: () => Arc,
	BezierCurve: () => BezierCurve,
	BoundingRect: () => BoundingRect,
	Circle: () => Circle,
	CompoundPath: () => CompoundPath,
	Ellipse: () => Ellipse,
	Group: () => Group,
	Image: () => ZRImage,
	IncrementalDisplayable: () => IncrementalDisplayable,
	Line: () => Line,
	LinearGradient: () => LinearGradient,
	Polygon: () => Polygon,
	Polyline: () => Polyline,
	RadialGradient: () => RadialGradient,
	Rect: () => Rect,
	Ring: () => Ring,
	Sector: () => Sector,
	Text: () => ZRText,
	clipPointsByRect: () => clipPointsByRect,
	clipRectByRect: () => clipRectByRect,
	createIcon: () => createIcon,
	extendPath: () => extendPath,
	extendShape: () => extendShape,
	getShapeClass: () => getShapeClass,
	getTransform: () => getTransform,
	initProps: () => initProps,
	makeImage: () => makeImage,
	makePath: () => makePath,
	mergePath: () => mergePath,
	registerShape: () => registerShape,
	resizePath: () => resizePath,
	updateProps: () => updateProps
});
//#endregion
//#region node_modules/echarts/lib/export/api/format.js
var format_exports = /* @__PURE__ */ __exportAll({
	addCommas: () => addCommas,
	capitalFirst: () => capitalFirst,
	encodeHTML: () => encodeHTML,
	formatTime: () => formatTime,
	formatTpl: () => formatTpl,
	getTextRect: () => getTextRect,
	getTooltipMarker: () => getTooltipMarker,
	normalizeCssArray: () => normalizeCssArray,
	toCamelCase: () => toCamelCase,
	truncateText: () => truncateText
});
//#endregion
//#region node_modules/echarts/lib/export/api/util.js
var util_exports = /* @__PURE__ */ __exportAll({
	bind: () => bind,
	clone: () => clone,
	curry: () => curry,
	defaults: () => defaults,
	each: () => each,
	extend: () => extend,
	filter: () => filter,
	indexOf: () => indexOf,
	inherits: () => inherits,
	isArray: () => isArray,
	isFunction: () => isFunction,
	isObject: () => isObject,
	isString: () => isString,
	map: () => map,
	merge: () => merge,
	reduce: () => reduce
});
//#endregion
//#region node_modules/echarts/lib/export/api.js
/**
* AUTO-GENERATED FILE. DO NOT MODIFY.
*/
function extendComponentModel(proto) {
	var Model = ComponentModel.extend(proto);
	ComponentModel.registerClass(Model);
	return Model;
}
function extendComponentView(proto) {
	var View = ComponentView.extend(proto);
	ComponentView.registerClass(View);
	return View;
}
function extendSeriesModel(proto) {
	var Model = SeriesModel.extend(proto);
	SeriesModel.registerClass(Model);
	return Model;
}
function extendChartView(proto) {
	var View = ChartView.extend(proto);
	ChartView.registerClass(View);
	return View;
}
//#endregion
export { Axis, ChartView, ComponentModel, ComponentView, SeriesData as List, Model, PRIORITY, SeriesModel, color_exports as color, connect, dataTool, dependencies, disConnect, disconnect, dispose, env, extendChartView, extendComponentModel, extendComponentView, extendSeriesModel, format_exports as format, getCoordinateSystemDimensions, getInstanceByDom, getInstanceById, getMap, graphic_exports as graphic, helper_exports as helper, init, brushSingle as innerDrawElementOnCanvas, matrix_exports as matrix, number_exports as number, parseGeoJSON, parseGeoJSON as parseGeoJson, registerAction, registerCoordinateSystem, registerLayout, registerLoading, registerLocale, registerMap, registerPostInit, registerPostUpdate, registerPreprocessor, registerProcessor, registerTheme, registerTransform, registerUpdateLifecycle, registerVisual, setCanvasCreator, setPlatformAPI, throttle, time_exports as time, use, util_exports as util, vector_exports as vector, version, util_exports$1 as zrUtil, zrender_exports as zrender };

//# sourceMappingURL=echarts_core.js.map