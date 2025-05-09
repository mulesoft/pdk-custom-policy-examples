


* Employee: 2182346 -> ClientID: cf3ad0ccc0914965a80a1ffde12fc5f8
* Admin: 2182345 -> ClientID: 8fd5e4d138c84b32a561b23718f99fa2

Admin
```
-y supergateway --sse https://demo-small-1-dqleu8.b8lv1r.usa-e2.cloudhub.io/mcp-flights/sse --header client_id:8fd5e4d138c84b32a561b23718f99fa2
```
Employee
```
-y supergateway --sse https://demo-small-1-dqleu8.b8lv1r.usa-e2.cloudhub.io/mcp-flights/sse --header client_id:cf3ad0ccc0914965a80a1ffde12fc5f8
```

```
permit(principal,action,resource) when { (resource == Tool::"make-reservation" || resource == Tool::"find-cheapest-flight")};
```

```
permit(principal,action,resource == Tool::"add-provider") when { principal.tier == "2182345"};
```

