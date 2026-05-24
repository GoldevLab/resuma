package main

import (
	"fmt"
	"net/http"
)

var count int

func main() {
	mux := http.NewServeMux()
	mux.Handle("/static/", http.StripPrefix("/static/", http.FileServer(http.Dir("static"))))
	mux.HandleFunc("/", func(w http.ResponseWriter, r *http.Request) {
		if r.Method != http.MethodGet {
			http.Error(w, "method not allowed", http.StatusMethodNotAllowed)
			return
		}
		CounterPage(count).Render(r.Context(), w)
	})
	mux.HandleFunc("/increment", func(w http.ResponseWriter, r *http.Request) {
		if r.Method != http.MethodPost {
			http.Error(w, "method not allowed", http.StatusMethodNotAllowed)
			return
		}
		count++
		CountFragment(count).Render(r.Context(), w)
	})
	fmt.Println("templ+htmx counter on :8080")
	_ = http.ListenAndServe(":8080", mux)
}
