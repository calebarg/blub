//
// blub_search.go
//
// Caleb Barger
// 12/02/2024
//

package main

//#cgo LDFLAGS: ./target/release/libblub_search.a -lm
//unsigned char blub_search_init();
//char* blub_search(char* query);
//void blub_search_give_back(char* search_results);
import "C"

import (
	"encoding/json"
	"fmt"
	"net/http"
	"net/url"
	"os"
	"strconv"
	"strings"
	"time"
)

type BlubSearchResult struct {
	Url   string `json:"url"`
	Title string `json:"title"`
	TitleSnippetHTML string  `json:"title_snippet_html"`
	BodySnippetHTML string  `json:"body_snippet_html"`
}

type IndexedDomainInfo struct {
	Domain       string `json:"domain"`
	PagesIndexed int `json:"pages_indexed"`
}

var globalIndexedDomainsRawHTML string = ""

func search(w http.ResponseWriter, req *http.Request) {
	queryParams, err := url.ParseQuery(req.URL.RawQuery)
	if err != nil {
		fmt.Fprint(w, err)
	}

	query, ok := queryParams["query"]
	var queryString = ""
	if ok {
		queryString = strings.Join(query, " ")
	}
	_, showIndexedDomains := queryParams["show_indexed_domains"]

	rawHTML := `<html><body><form action="/">
								<input type="text" name="query"/><input type="submit" value="Blub Search">
								<input type="submit" name="show_indexed_domains" value="Indexed domains info"/>`
	if showIndexedDomains {
		rawHTML += globalIndexedDomainsRawHTML
	} else if len(queryString) > 0 { // Otherwise search results
		searchStart := time.Now()
		result_cstr := C.blub_search(C.CString(queryString))
		searchTimeElapsed := time.Since(searchStart)
		searchLatencyMS := searchTimeElapsed.Milliseconds()

		result := C.GoString(result_cstr)
		C.blub_search_give_back(result_cstr) // NOTE(calebarg): Be careful make sure go gets a copy.

		var searchResults []BlubSearchResult
		json.Unmarshal([]byte(result), &searchResults)

		rawHTML += "<p>Search latency " + strconv.Itoa(int(searchLatencyMS)) + "ms</p>"
		rawHTML += "</ul>"
		for resultIdx := 0; resultIdx < len(searchResults); resultIdx++ {
			rawHTML += "<li>"

			searchResult := searchResults[resultIdx]
			rawHTML += "<a href=\"" + searchResult.Url + "\">" + searchResult.Url + "</a>"
			rawHTML += "<span> -- " + searchResult.Title + "</span>"
			rawHTML += "<div>" + searchResult.BodySnippetHTML + "</div>"

			rawHTML += "</li>"
		}
		rawHTML += "</ul>"
	}
	rawHTML += "</body></html>"
	fmt.Fprint(w, rawHTML)
}

func main() {
	if C.blub_search_init() == 0 {
		panic("Failed to initialize blub search")
	}

	indexed_domains_info_contents, err := os.ReadFile("blub2-data/indexed_domains_info.json")
	if err == nil {
		var indexedDomainsInfo []IndexedDomainInfo
		json.Unmarshal(indexed_domains_info_contents, &indexedDomainsInfo)
		globalIndexedDomainsRawHTML += "<ul>"
		for indexedDomainIdx := 0; indexedDomainIdx < len(indexedDomainsInfo); indexedDomainIdx++ {
			indexedDomain := indexedDomainsInfo[indexedDomainIdx]
			globalIndexedDomainsRawHTML += "<li><p>" + indexedDomain.Domain + " pages indexed " + strconv.Itoa(indexedDomain.PagesIndexed) + "</p></li>"
		}
		globalIndexedDomainsRawHTML += "</ul>"
	} else {
		fmt.Println("!!Failed read indexexd_domains_info.json!!")
	}

	http.HandleFunc("/", search)
	http.ListenAndServe(":8090", nil)
}
