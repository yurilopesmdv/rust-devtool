package example

import (
	"context"
	"encoding/json"
	"net/http"
)

type Reader interface {
	Get(context.Context) (Example, error)
}
type Controller struct{ Service Reader }

func (c Controller) Get(w http.ResponseWriter, r *http.Request) {
	result, err := c.Service.Get(r.Context())
	if err != nil {
		http.Error(w, "Falha ao consultar exemplo", http.StatusInternalServerError)
		return
	}
	w.Header().Set("Content-Type", "application/json")
	_ = json.NewEncoder(w).Encode(result)
}
