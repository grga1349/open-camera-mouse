export namespace config {
	
	export class Params {
	    templateSizePx: number;
	    gainMultiplier: number;
	    smoothing: number;
	    dwellEnabled: boolean;
	    dwellTimeMs: number;
	    autoStart: boolean;
	    rightClickEnabled: boolean;
	
	    static createFrom(source: any = {}) {
	        return new Params(source);
	    }
	
	    constructor(source: any = {}) {
	        if ('string' === typeof source) source = JSON.parse(source);
	        this.templateSizePx = source["templateSizePx"];
	        this.gainMultiplier = source["gainMultiplier"];
	        this.smoothing = source["smoothing"];
	        this.dwellEnabled = source["dwellEnabled"];
	        this.dwellTimeMs = source["dwellTimeMs"];
	        this.autoStart = source["autoStart"];
	        this.rightClickEnabled = source["rightClickEnabled"];
	    }
	}

}

