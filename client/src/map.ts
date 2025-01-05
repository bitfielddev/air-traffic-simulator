import { ref } from "vue";
import config from "./config";

export const map = ref<L.Map>();

// @ts-ignore
// https://stackoverflow.com/questions/73542576/leaflet-error-when-zooming-after-closing-popup
function leafletZoomFix() {
  L.Tooltip.prototype._animateZoom = function (opt) {
    if (!this._map) {
      return;
    }
    const pos = this._map._latLngToNewLayerPoint(
      this._latlng,
      opt.zoom,
      opt.center,
    );
    this._setPosition(s);
  };
}

export function initMap() {
  leafletZoomFix();

  map.value = L.map("map", {
    preferCanvas: true,
    crs: L.CRS.Simple,
  }).setView([0, 0], 0);

  config.tileLayer.addTo(map.value);
}
