export namespace config {
	
	export class GeneralParams {
	    autoStart: boolean;
	    dwellOnStartup: boolean;
	
	    static createFrom(source: any = {}) {
	        return new GeneralParams(source);
	    }
	
	    constructor(source: any = {}) {
	        if ('string' === typeof source) source = JSON.parse(source);
	        this.autoStart = source["autoStart"];
	        this.dwellOnStartup = source["dwellOnStartup"];
	    }
	}
	export class HotkeysParams {
	    startPause: string;
	    recenter: string;
	
	    static createFrom(source: any = {}) {
	        return new HotkeysParams(source);
	    }
	
	    constructor(source: any = {}) {
	        if ('string' === typeof source) source = JSON.parse(source);
	        this.startPause = source["startPause"];
	        this.recenter = source["recenter"];
	    }
	}
	export class ClickingParams {
	    dwellEnabled: boolean;
	    dwellTimeMs: number;
	    dwellRadiusPx: number;
	    clickType: string;
	    rightClickToggle: boolean;
	
	    static createFrom(source: any = {}) {
	        return new ClickingParams(source);
	    }
	
	    constructor(source: any = {}) {
	        if ('string' === typeof source) source = JSON.parse(source);
	        this.dwellEnabled = source["dwellEnabled"];
	        this.dwellTimeMs = source["dwellTimeMs"];
	        this.dwellRadiusPx = source["dwellRadiusPx"];
	        this.clickType = source["clickType"];
	        this.rightClickToggle = source["rightClickToggle"];
	    }
	}
	export class PointerAdvancedParams {
	    gainX: number;
	    gainY: number;
	    smoothing: number;
	
	    static createFrom(source: any = {}) {
	        return new PointerAdvancedParams(source);
	    }
	
	    constructor(source: any = {}) {
	        if ('string' === typeof source) source = JSON.parse(source);
	        this.gainX = source["gainX"];
	        this.gainY = source["gainY"];
	        this.smoothing = source["smoothing"];
	    }
	}
	export class PointerParams {
	    sensitivity: number;
	    deadzonePx: number;
	    maxSpeedPx: number;
	    advanced?: PointerAdvancedParams;
	
	    static createFrom(source: any = {}) {
	        return new PointerParams(source);
	    }
	
	    constructor(source: any = {}) {
	        if ('string' === typeof source) source = JSON.parse(source);
	        this.sensitivity = source["sensitivity"];
	        this.deadzonePx = source["deadzonePx"];
	        this.maxSpeedPx = source["maxSpeedPx"];
	        this.advanced = this.convertValues(source["advanced"], PointerAdvancedParams);
	    }
	
		convertValues(a: any, classs: any, asMap: boolean = false): any {
		    if (!a) {
		        return a;
		    }
		    if (a.slice && a.map) {
		        return (a as any[]).map(elem => this.convertValues(elem, classs));
		    } else if ("object" === typeof a) {
		        if (asMap) {
		            for (const key of Object.keys(a)) {
		                a[key] = new classs(a[key]);
		            }
		            return a;
		        }
		        return new classs(a);
		    }
		    return a;
		}
	}
	export class TrackingParams {
	    templateSizePx: number;
	    searchMarginPx: number;
	    scoreThreshold: number;
	    adaptiveTemplate: boolean;
	    templateUpdateAlpha: number;
	    markerShape: string;
	
	    static createFrom(source: any = {}) {
	        return new TrackingParams(source);
	    }
	
	    constructor(source: any = {}) {
	        if ('string' === typeof source) source = JSON.parse(source);
	        this.templateSizePx = source["templateSizePx"];
	        this.searchMarginPx = source["searchMarginPx"];
	        this.scoreThreshold = source["scoreThreshold"];
	        this.adaptiveTemplate = source["adaptiveTemplate"];
	        this.templateUpdateAlpha = source["templateUpdateAlpha"];
	        this.markerShape = source["markerShape"];
	    }
	}
	export class AllParams {
	    tracking: TrackingParams;
	    pointer: PointerParams;
	    clicking: ClickingParams;
	    hotkeys: HotkeysParams;
	    general: GeneralParams;
	
	    static createFrom(source: any = {}) {
	        return new AllParams(source);
	    }
	
	    constructor(source: any = {}) {
	        if ('string' === typeof source) source = JSON.parse(source);
	        this.tracking = this.convertValues(source["tracking"], TrackingParams);
	        this.pointer = this.convertValues(source["pointer"], PointerParams);
	        this.clicking = this.convertValues(source["clicking"], ClickingParams);
	        this.hotkeys = this.convertValues(source["hotkeys"], HotkeysParams);
	        this.general = this.convertValues(source["general"], GeneralParams);
	    }
	
		convertValues(a: any, classs: any, asMap: boolean = false): any {
		    if (!a) {
		        return a;
		    }
		    if (a.slice && a.map) {
		        return (a as any[]).map(elem => this.convertValues(elem, classs));
		    } else if ("object" === typeof a) {
		        if (asMap) {
		            for (const key of Object.keys(a)) {
		                a[key] = new classs(a[key]);
		            }
		            return a;
		        }
		        return new classs(a);
		    }
		    return a;
		}
	}
	
	
	
	
	

}

