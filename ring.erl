-module(ring).
-export([main/1, manager/1]).

-ifdef(DEBUG).
-define(DBG(F, A), io:format(F, A)).
-else.
-define(DBG(_F, _A), ok).
-endif.
-define(MAX, 3).
main(_) ->
    Tests = [{round(math:pow(10, Procs)), round(math:pow(10, Msgs)), round(math:pow(2, Paralell)), round(math:pow(10, Size))} ||
        Procs <- lists:seq(1, ?MAX),
        Msgs <- lists:seq(1, ?MAX),
        Paralell <- lists:seq(0, ?MAX - 1),
        Size <- lists:seq(1, ?MAX - 1)
        ],
    manager(Tests),
    receive
        done -> ok
    end.

setup(Ref, Procs, Msgs) ->
    spawn(fun () -> init_first(Ref, Procs, Msgs) end).


init_first(Ref, Procs, Msgs) ->
    First = self(),
    Next = spawn(fun() -> init(Procs - 1, Ref, Msgs, First) end),
    loop(Msgs, Ref, Next).


init(0, Ref, Msgs, First) ->
    manager ! {init_done, Ref, erlang:system_time()},
    loop(Msgs, Ref, First);

init(Procs, Ref, Msgs, First) ->
    ?DBG("[~10p] Starting process ~p.\n", [Procs, self()]),
    Next = spawn(fun() -> init(Procs - 1, Ref, Msgs, First) end),
    loop(Msgs, Ref, Next).

loop(X, Ref, Next) when X < 0->
    ?DBG("[~p] done\n", [self()]),
    try
        manager ! {loop_done, Ref, erlang:system_time()}
    catch
        _ -> ok
    end,
    Next ! exit;

loop(Msgs, Ref, Next) ->
    receive
        {msg, M} ->
            ?DBG("[~p] msg ~p\n", [self(), Msgs]),
            Next ! {msg, M},
            loop(Msgs - 1, Ref, Next);
        exit ->
            ?DBG("[~p] done (~p)\n", [self(), Msgs]),
            ok
    end.

manager(Tests) ->
    Main = self(),
    spawn(fun() -> manager_loop(Tests), Main ! done end).
manager_loop(Tests) ->
    register(manager, self()),
    io:format("Running ~p Tests ", [length(Tests)]),
    manager_loop_1(Tests, []).

write_results(_F, []) ->
    ok;
write_results(F, [{{Procs, Messages, Paralell, Size}, TimeInit, TimeLoop} | R]) ->
    file:write(F, io_lib:format("erlang,~p,~p,~p,~p,~p,~p\n", [Procs, Messages, Paralell, Size, TimeInit, TimeLoop])),
    write_results(F, R).

manager_loop_1([], Results) ->
    {ok, F} = file:open("results/results.erlang", [write]),
    write_results(F, lists:reverse(Results)),
    file:close(F),
    io:format(" All tests done\n");
manager_loop_1([{Procs, Messages, Paralell, Size} = C | Tests], Results) ->
    io:format("\nRunning ~p procs with ~p messages ~p in paralell of a size of ~p", [Procs, Messages, Paralell, Size]),
    Ref = make_ref(),
    Start = erlang:system_time(),
    Proc = setup(Ref, Procs, Messages),
    receive
        {init_done, Ref, End} ->
            flush(),
            T = manager_loop_2(Ref, C, Proc),
            io:format("."),
            manager_loop_1(Tests, [{C, End - Start, T} | Results])
    end.
manager_loop_2(Ref, {_Procs, _Messages, Paralell, Size}, Proc) ->
    Data = <<0:(Size*8)>>,
    Start = erlang:system_time(),
    messages(Paralell, Proc, Data),
    receive
        {loop_done, Ref, End} ->
            End - Start
    end.
messages(0, _, _) ->
    ok;
messages(N, Proc, Data) ->
    Proc ! {msg, Data},
    messages(N - 1, Proc, Data).

flush() ->
    receive
        _ -> flush()
    after
        0 -> ok
    end.